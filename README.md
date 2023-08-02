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
        |    Blockchain   |<--+        +----------------+
        | Smart Contracts |   |        |+----+---------+|
        +-----------------+   +--------||  Middleware  ||
                                       |+------+-------+|
                                       |       |        |
                                       |       |        |
                                       |+------+-----+  |
                                       ||   Cache    |  |
                                       |+------------+  |
                                       +----------------+
                                             SERVER
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
All the command are assuming you are in the project's root folder
(ie /home/.../caramell)


### mqtt
runing the mqtt: 
`docker pull bytebeamio/rumqttd`
`docker run -d -p 1883:1883 -p 1884:1884 -it --name rumqttd bytebeamio/rumqttd`  

or use anything that tickle your fancy (mosquitto/rmqtt...).  
I'd recommend using mosquitto, since it is the most stable and well polished.
It has some feature like a load balancer in a round robin fashion to deliver 
messages.

But, for demonstration purpose, none of this is needed.

---
### cacher
running memcached:  
`docker pull memcached`
`docker run --name memcache -d -p 11211:11211 memcached -m20000m -I500m`  
where `m<size>` indicates the max size allocated to memcached, and
`-I<size>` the max item size.

---

### running the blockchain:  
you will have to this project's (caramell) root and ensure that you have 
also pulled git submodules: `git submodules init ; git submodule update`

Then, execute the init script, and the blockchain nodes:
`cd caramell-blockchain ; sh blockchainInit/init.sh ; docker compose up`


client: `cargo run --bin caramell-client`  
server: `cargo run --bin caramell-server`  

---

### Front End
there is a small interface written **very** poorly. To run it, put yourself in
its root directory with `cd ./caramell-frontend` and run it with
`npm install ; npm run dev`. Please note that it is still quite broken, and 
does not have all func it is supposed to have. i'll update it when i have more 
time.
