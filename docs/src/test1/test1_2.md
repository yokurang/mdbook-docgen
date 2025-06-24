# 02 – High-Level Architecture

Venenatis lacinia lup surm, tempor gravida justo blandit quis.  
Morbi posuere, sapien eget condimentum maximus, felis leo maximus risus.

```mermaid
flowchart LR
    subgraph Frontend
        A[Web App]
    end
    subgraph Backend
        B(API Gateway) --> C(Auth Service)
        C --> D[Payments Service]
    end
    A -->|HTTPS| B
````

```plantuml
@startuml
cloud "Users"            as U
node  "API-Gateway"      as G
component "Auth-svc"     as A
component "Payments-svc" as P
database  "Ledger-DB"    as L

U -> G : REST
G --> A : JWT verify
G --> P
P --> L : SQL
@enduml
```

```bash
# deploy.sh
docker compose up -d --build
```