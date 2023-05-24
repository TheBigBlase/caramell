# Cache And Ressource / Assets Manager, Efficient and Low Level 
caramell (that's a big stretch for a funni name)

This is a research project, inside the [Parfait project](https://anr.fr/Project-ANR-21-CE25-0013) (they stretched further to get their name, im safe)  

This project aims to bring cost calculation mechanism to a caching system.
A user can store some data to cache, if and only if he brings a service of 
equivalent cost, or if he pays for it.  
The caching system needs to be able to scale / be distributed

## Structure
```
client --> broker (MoM) --> cache system
                  | |
                  | |  (broker is
                  | |  the one doing the
                  | |       accounting)
                  \ /
                   +
              accounting
```

Notes:
* the broker is a MQTT (for now). 
    * pro:
        * lightweight
        * easy to use / hard to missuse
        * topics can be persistent
    * using rumqtt, any MQTT implementation works
* cache
    * aims to be cache agnostic.
    * using memcached in the meantime.
    * 
* accounting
    * PoA blockchain (recommanded by my suppervisors).
    * besu hyperledger

## usage
runing the mqtt: 
`docker pull bytebeamio/rumqttd
docker run -d -p 1883:1883 -p 1884:1884 -it --name rumqttd bytebeamio/rumqttd`  

running memcached: 
`docker pull memcached
docker run --name memcache -d -p 11211:11211 memcached`  

client: `cargo run --bin caramell-client`  
server: `cargo run --bin caramell-server`  

## TODO
* for now client sends dummy data, and server inserts it into memcache
* gotta do client better
* finish base of server
* do server => blockchain
* mqqt => blockchain as well?
* calc costs
