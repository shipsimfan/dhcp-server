# Hart DHCP Server
An simple to configure cross-platform DHCP server written in Rust with a built
in HTTP server for easy diagnostics.

## Building
The server is built using [cargo](https://www.rust-lang.org/) 

```sh
cargo build --release
```

## Running
The server executable can be run directly. One argument may be passed in to
specify the location of a configuration file. If not specified, the server will
look for a file named 'config' in the working directory.

## Configuration
The server has many options for configuring the server. Many of which are
required. An example configuration file is provided for syntax reference.
Further information on the syntax of the configuration file can be found at
the respository for [lib-config](https://www.github.com/shipsimfan/lib-config).

The following options are required:
 1. lease.start - The first address to be given out as a lease.
 2. lease.final - The last address to be given out as a lease, all leased I.P.
    addresses will be between lease.start and lease.end (inclusively). The
    server will not hand out I.P. address that follow either x.x.x.255 or 
    x.x.x.0.
 3. gateway - The I.P. address of the network's gateway.
 4. us - The I.P. address assigned to the DHCP server.
 5. subnet mask - The network's subnet mask.
 6. broadcast - The network's broadcast I.P. address.
 7. dns.1 - The primary I.P. address of the network's DNS server.
 8. dns.2 - The alternative I.P. address of the network's DNS server.
 
The following options are optional:
 1. reserved - An array holding reserved I.P. addresses and their corrosponding
    MAC address. Defaults to an empty array.
 2. lease.time - The number of seconds a lease should last for. Defaults to 
    172 800 seconds or 2 days.
 3. renewal time - The number of seconds before a client should renew their
    lease. Defaults to half of lease.time.
 4. rebinding time - The number of seconds before a client should attempt to
    rebind their lease. Defaults to 3/4 of lease.time.
 5. offer time - The number of seconds an offer for an I.P. address should
    last. Defaults to 30 seconds.
 6. log limit - The maximum number of logs the HTTP server should hold on to
    and display. Defaults to no limit.
 7. log - The location of a log file. Defaults to no log file.