# Zenoh-net Python examples

## Start instructions

   ```bash
   python3 <example.py>
   ```

   Each example accepts the `-h` or `--help` option that provides a description of its arguments and their default values.

   If you run the tests against the zenoh router running in a Docker container, you need to add the
   `-e tcp/localhost:7447` option to your examples. That's because Docker doesn't support UDP multicast
   transport, and therefore the zenoh scouting and discrovery mechanism cannot work with.

## Examples description

### z_scout

   Scouts for zenoh peers and routers available on the network.

   Typical usage:
   ```bash
      python3 z_scout.py
   ```

### z_info

   Gets information about the zenoh-net session.

   Typical usage:
   ```bash
      python3 z_info.py
   ```


### z_write

   Writes a path/value into Zenoh.  
   The path/value will be received by all matching subscribers, for instance the [z_sub](#z_sub)
   and [z_storage](#z_storage) examples.

   Typical usage:
   ```bash
      python3 z_write.py
   ```
   or
   ```bash
      python3 z_write.py -p /demo/example/test -v 'Hello World'
   ```

### z_pub

   Declares a resource with a path and a publisher on this resource. Then writes a value using the numerical resource id.
   The path/value will be received by all matching subscribers, for instance the [z_sub](#z_sub)
   and [z_storage](#z_storage) examples.

   Typical usage:
   ```bash
      python3 z_pub.py
   ```
   or
   ```bash
      python3 z_pub.py -p /demo/example/test -v 'Hello World'
   ```

### z_sub

   Registers a subscriber with a selector.  
   The subscriber will be notified of each write made on any path matching the selector,
   and will print this notification.

   Typical usage:
   ```bash
      python3 z_sub.py
   ```
   or
   ```bash
      python3 z_sub.py -s /demo/**
   ```

### z_pull

   Registers a pull subscriber with a selector.  
   The pull subscriber will receive each write made on any path matching the selector,
   and will pull on demand and print the received path/value.

   Typical usage:
   ```bash
      python3 z_pull.py
   ```
   or
   ```bash
      python3 z_pull.py -s /demo/**
   ```

### z_query

   Sends a query message for a selector.  
   The queryables with a matching path or selector (for instance [z_eval](#z_eval) and [z_storage](#z_storage))
   will receive this query and reply with paths/values that will be received by the query callback.

   Typical usage:
   ```bash
      python3 z_query.py
   ```
   or
   ```bash
      python3 z_query.py -s /demo/**
   ```

### z_eval

   Registers a queryable function with a path.  
   This queryable function will be triggered by each call to a query operation on zenoh-net
   with a selector that matches the path, and will return a value to the querier.

   Typical usage:
   ```bash
      python3 z_eval.py
   ```
   or
   ```bash
      python3 z_eval.py -p /demo/example/eval -v 'This is the result'
   ```

### z_storage

   Trivial implementation of a storage in memory.  
   This examples registers a subscriber and a queryable on the same selector.
   The subscriber callback will store the received paths/values in an hashmap.
   The queryable callback will answer to queries with the paths/values stored in the hashmap
   and that match the queried selector.

   Typical usage:
   ```bash
      python3 z_storage.py
   ```
   or
   ```bash
      python3 z_storage.py -s /demo/**
   ```

### z_pub_thr & z_sub_thr

   Pub/Sub throughput test.
   This example allows to perform throughput measurements between a pubisher performing
   write operations and a subscriber receiving notifications of those writes.

   Typical Subscriber usage:
   ```bash
      python3 z_sub_thr.py
   ```

   Typical Publisher usage:
   ```bash
      python3 z_pub_thr.py 1024
   ```