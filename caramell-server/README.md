# Cache And Ressource or Assets Manager, Efficient and Low Latency
(i really went too far to get this name xd)

## This is for research purpose.
Caramell is a distributed cache manager. It allows one to store data on a
distributed node, and retrive it. Each interaction will be recoreded, and a cost
will be associated with it. A user can either pay, or offer equivalent service. 

## Design
This project uses memcached (for now) as a caching system, and rumqtt as a
message broker. 

Basic schema:
```
client --> broker (MOM) --> cache system
                  | |
                  | |  (broker is 
                  | |  the one doing the
                  | |         accounting)
                  \ /
                   +
               accounting
```

Accounting will be done using a PoA blockchain. 

## TODO 
* broker / cache interaction
* broker / accounting
* client / broker
* cost calc
* client -> client interaction
