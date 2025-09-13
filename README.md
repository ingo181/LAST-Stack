# LAST-Stack
This is the last stack you need.
The stack consists of the following components:
- **L**eptos
- **A**xum
- **S**urrealDB
- **T**auri
- Thaw

The microservice architecture with docker/podman:
- SurrealDB-database instance
- Apache Kafka
- Apache Zookeeper

  ## Block diagram
  <img width="1568" height="367" alt="image" src="https://github.com/user-attachments/assets/f9fcac30-d06c-4e27-be3b-f35c55702a5f" />

## Data flow diagram
<img width="240" height="516" alt="Pasted image 20250606174812" src="https://github.com/user-attachments/assets/268875e8-bbc7-43f5-bd06-db99511935fd" />

The diagram shows the data flow from the request to data storage:

- The client sends requests via HTTP to the Axum router
- The authentication layer checks access rights and tokens
- SurrealDB processes the authorized requests
- The data is stored in the selected storage system (e.g., SurrealKV for local development)
