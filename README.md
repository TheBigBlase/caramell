# Cache And Ressource / Assets Manager, Efficient and Low Level 
caramell (that's a big stretch for a funni name)

This is a research project, inside the [Parfait project](https://anr.fr/Project-ANR-21-CE25-0013) (they stretched further to get their name, im safe)  

This project aims to bring cost calculation mechanism to a caching system.
A user can store some data to cache, if and only if he brings a service of 
equivalent cost, or if he pays for it.  
The caching system needs to be able to scale / be distributed

## Structure
```
   O     +----------+       +----------+
  /|\ >->|  Client  |------>|   MoM    |\
   ^     +-----+----+       +----------+ \
  / \          |                          \
  User  +------+----------+                \
        |    Blockchain   |<--+             \
        | Smart Contracts |   |         +----+---------+
        +-----------------+   +-------->|  Middleware  |
                                        +------+-------+
                                               |
                                               |
                                        +------+-----+  
                                        |   Cache    |  
                                        +------------+  
```

Notes:
* the Mom is MQTT compliant (for now). 
    * pro:
        * lightweight
        * easy to use / hard to missuse
        * topics can be persistent
    * using rumqtt, any MQTT implementation works
* cache
    * aims to be cache agnostic.
    * using memcached in the meantime.
* accounting
    * PoA blockchain (recommanded by my suppervisors).
    * besu hyperledger
* MW:
	* acts for off chain verif / actions
	* is a overhead of cache

## usage
runing the mqtt: 
`docker pull bytebeamio/rumqttd
docker run -d -p 1883:1883 -p 1884:1884 -it --name rumqttd bytebeamio/rumqttd`  

or use anything you fancy (mosquitto/rmqtt...)

running memcached: 
`docker pull memcached
docker run --name memcache -d -p 11211:11211 memcached`  

For running rust components, you'll need my fork of ethers-rs, unless it has been 
merged in upstream. Ethers.rs will need to be in the same directory as this one is,
install it like so:
`cd ../ && git clone https://github.com/thebigblase/ethers-rs && cd caramell`

running the blockchain:
`cd caramell-blockchain ; sh blockchainInit/init.sh ; docker compose up`

client: `cargo run --bin caramell-client`  
server: `cargo run --bin caramell-server`  

## TODO
* for now client sends dummy data, and server inserts it into memcache
* gotta do client better
* ~~finish base of server~~
* ~~do server => blockchain~~
* ~~mqtt => blockchain as well?~~
* paying mechanism: contract witholding (?) gas, then retrievable by cache owner
	* contract side
	* cacher side
	* client side
