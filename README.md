# LeakChef

Hello there and welcome to the README of my Bachelor's thesis. At least the README of the implementation. So here is a quick guide of where what is. 

- `docker-compose.yml`: What I use to spin up my containers. Three services are listed there: the server (leakchef), the proxy and the database. If you want the client, then please look [here](./demo-client/README.md).
- `openapi.yml`: essentially the protocol
- [server](./server/): Also known as the LeakChef. Here is where the heavy-lifting takes place, such as handling requests, communicating directly with the database etc. This is a Rust-based webserver developed with Actix 🦀. MongoDB was used as a database. The server was designed with extensibility in mind, so the DB interface and/or the cache optimizations can easily be exchanged according to the application requirements.
- [lib-client](./lib-client/): This is the client library. It conveniently wraps calls to functions of the server and processes the response.
- [demo-client](./demo-client/): This is the client stub. Here, some basic routines are demonstrated. This component is a binary and is meant to be used like one.
- [proxy](./proxy/): Here is the NGINX proxy configuration.

If you want to understand that component better, please consult that folder's readme or the code documentation.

## Q: Wait, so what exactly was your thesis about?

The goal of this work was designing and implementing a protocol for a client-server application to mitigate credential stuffing. The foundation of this research encompasses the groundwork for addressing data leaks on a broader scope and includes an examination of the implications of credential stuffing attacks. This foundation is further enhanced by the principles of effective protocol design. Furthermore, contextualization is provided by related C3 services. It should be noted that this thesis explicitly delineates clear boundaries between the aforementioned foundational knowledge and the specific contributions of this thesis. This demarcation ensures a clearly defined scope for the current project. To achieve this objective, a comprehensive checklist for the system was carefully compiled in collaboration with Pascua Theus from Identeco GmbH & Co. KG. Using these requirements as a design framework, the protocol was created in a systematic manner. The protocol specification incorporates a combination of diverse components, including the following:

- The OAS was used to formally define the service primitives and specify the messages that will be exchanged between communication partners, guaranteeing a standardised and well-documented interface for effortless interaction. 
- A detailed written language approach was used to explain the intricacies of interactions, the flow of events between peers and the responses to various messages. 
- With regards to security, different threats were thoroughly discussed and included in the overall design. Consequently, the design was equipped to withstand potential security threats in a robust and resilient manner.


This multi-faceted approach to design ensures that the system fulfils functional requirements, as well as addressing security and interoperability concerns, making it a strong and valuable component within the Identeco GmbH & Co. KG ecosystem. Afterwards, the system was implemented according to the design. Any required technical details were added to ensure reproducibility. Moreover, unit and integration tests were composed to ensure a precise and appropriate implementation. Finally, an objective evaluation was performed on the system. In order to achieve this, the requirements were iterated and their degree of fulfilment was analysed. Furthermore, the system was benchmarked to provide a comprehensive overview of its capabilities and limitations. This yielded valuable insights into the speed at which leaks are downloaded by the client application. In addition, the memory consumption of both the client and server - essential components of the system - was also taken into account. Based on the analysis conducted in chapter 6, it can be stated with confidence that all requirements were completely met, thereby achieving the objective of this thesis.
