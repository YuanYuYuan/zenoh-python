//
// Copyright (c) 2026 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use std::sync::Arc;

use pyo3::{prelude::*, types::PyBytes};
use zenoh::timestamp_stack::{
    InstrumentationTimestamp, InterceptionPoint as RustInterceptionPoint, SessionTimestampCallback,
    TimestampInstrumentationBuilder as RustTimestampInstrumentationBuilder,
    TsStackContext as RustTsStackContext,
};

use crate::{
    config::{WhatAmI, ZenohId},
    macros::wrapper,
    time::Timestamp,
    utils::IntoPyResult,
};

// ── TimestampInstrumentationBuilder ──────────────────────────────────────────

#[pyclass]
pub(crate) struct TimestampInstrumentationBuilder {
    send: bool,
    route: bool,
    receive: bool,
}

#[pymethods]
impl TimestampInstrumentationBuilder {
    #[new]
    fn new() -> Self {
        Self {
            send: false,
            route: false,
            receive: false,
        }
    }

    fn set_send(mut self_: PyRefMut<Self>, send: bool) -> PyRefMut<Self> {
        self_.send = send;
        self_
    }

    fn set_route(mut self_: PyRefMut<Self>, route: bool) -> PyRefMut<Self> {
        self_.route = route;
        self_
    }

    fn set_receive(mut self_: PyRefMut<Self>, receive: bool) -> PyRefMut<Self> {
        self_.receive = receive;
        self_
    }

    fn build(&self) -> PyResult<TimestampInstrumentation> {
        RustTimestampInstrumentationBuilder::new()
            .set_send(self.send)
            .set_route(self.route)
            .set_receive(self.receive)
            .build()
            .map(TimestampInstrumentation)
            .into_pyres()
    }

    fn __repr__(&self) -> String {
        format!(
            "TimestampInstrumentationBuilder(send={}, route={}, receive={})",
            self.send, self.route, self.receive
        )
    }
}

// InterceptionPoint is #[non_exhaustive] so we can't use enum_mapper! (it generates exhaustive
// From impls). Define it manually with a repr u8 for Python comparison, and a fallback variant.
#[pyo3::pyclass(eq)]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InterceptionPoint {
    #[pyo3(name = "SEND")]
    Send = 0,
    #[pyo3(name = "ROUTE")]
    Route = 1,
    #[pyo3(name = "RECEIVE")]
    Receive = 2,
    /// Catch-all for future variants added by the Rust core.
    #[pyo3(name = "UNKNOWN")]
    Unknown = 255,
}

impl From<RustInterceptionPoint> for InterceptionPoint {
    fn from(v: RustInterceptionPoint) -> Self {
        match v {
            RustInterceptionPoint::Send => Self::Send,
            RustInterceptionPoint::Route => Self::Route,
            RustInterceptionPoint::Receive => Self::Receive,
        }
    }
}

impl From<InterceptionPoint> for RustInterceptionPoint {
    fn from(v: InterceptionPoint) -> Self {
        match v {
            InterceptionPoint::Send => RustInterceptionPoint::Send,
            InterceptionPoint::Route => RustInterceptionPoint::Route,
            InterceptionPoint::Receive | InterceptionPoint::Unknown => {
                RustInterceptionPoint::Receive
            }
        }
    }
}

wrapper!(zenoh::timestamp_stack::TsStackContext: Clone);

#[pymethods]
impl TsStackContext {
    #[getter]
    fn zid(&self) -> ZenohId {
        self.0.zid.into()
    }

    #[getter]
    fn whatami(&self) -> WhatAmI {
        self.0.whatami.into()
    }

    #[getter]
    fn interception_point(&self) -> InterceptionPoint {
        self.0.interception_point.into()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

wrapper!(zenoh::timestamp_stack::TimestampInstrumentation: Clone, Copy);

#[pymethods]
impl TimestampInstrumentation {
    #[new]
    #[pyo3(signature = (*, send = false, route = false, receive = false))]
    fn new(send: bool, route: bool, receive: bool) -> PyResult<Self> {
        RustTimestampInstrumentationBuilder::new()
            .set_send(send)
            .set_route(route)
            .set_receive(receive)
            .build()
            .map(Self)
            .into_pyres()
    }

    fn is_instrumented(&self, point: InterceptionPoint) -> bool {
        self.0.is_instrumented(point.into())
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

wrapper!(zenoh::timestamp_stack::TimestampStackRecord: Clone);

#[pymethods]
impl TimestampStackRecord {
    #[getter]
    fn point(&self) -> InterceptionPoint {
        self.0.point().into()
    }

    #[getter]
    fn is_custom(&self) -> bool {
        self.0.is_custom()
    }

    fn timestamp(&self, py: Python) -> PyResult<PyObject> {
        match self.0.timestamp() {
            InstrumentationTimestamp::UHLC(ts) => {
                Ok(Timestamp(*ts).into_pyobject(py)?.into_any().unbind())
            }
            InstrumentationTimestamp::Custom(bytes) => {
                Ok(PyBytes::new(py, bytes).into_any().unbind())
            }
        }
    }

    fn as_timestamp(&self) -> Option<Timestamp> {
        match self.0.timestamp() {
            InstrumentationTimestamp::UHLC(ts) => Some(Timestamp(*ts)),
            InstrumentationTimestamp::Custom(_) => None,
        }
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

wrapper!(zenoh::timestamp_stack::TimestampStack: Clone);

#[pymethods]
impl TimestampStack {
    #[getter]
    fn instrumentation(&self) -> TimestampInstrumentation {
        TimestampInstrumentation(self.0.instrumentation())
    }

    #[getter]
    fn records(&self) -> Vec<TimestampStackRecord> {
        self.0
            .records()
            .iter()
            .cloned()
            .map(TimestampStackRecord)
            .collect()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

/// Build a `SessionTimestampCallback` Arc from a Python callable.
pub(crate) fn py_to_session_ts_callback(py_cb: PyObject) -> SessionTimestampCallback {
    Arc::new(move |ctx: RustTsStackContext| {
        Python::with_gil(|py| {
            let py_ctx = match Py::new(py, TsStackContext(ctx)) {
                Ok(obj) => obj,
                Err(_) => return Vec::new(),
            };
            match py_cb.call1(py, (py_ctx,)) {
                Ok(result) => result.extract::<Vec<u8>>(py).unwrap_or_default(),
                Err(e) => {
                    if let Ok(logging) = py.import("logging") {
                        if let Ok(logger) = logging.call_method1("getLogger", ("zenoh",)) {
                            let _ = logger
                                .call_method1("error", (format!("Timestamp callback error: {e}"),));
                        }
                    }
                    Vec::new()
                }
            }
        })
    })
}
