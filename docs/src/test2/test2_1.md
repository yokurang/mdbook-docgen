# 03 – Key Sequence Flows

Integer faucibus lup surm arcu, vel sagittis est porta sed.  
Nullam dignissim, justo sed facilisis volutpat, dolor turpis dictum est.

```mermaid
sequenceDiagram
    actor User
    participant FE as Frontend
    participant BE as Backend
    participant DB as Database

    User->>FE: Click “Pay”
    FE->>BE: POST /payments
    BE->>DB: INSERT payment
    DB-->>BE: OK
    BE-->>FE: 201 Created
    FE-->>User: Payment confirmed
````

```plantuml
@startuml
title Refund workflow
participant "Client App" as C
participant "Payments-svc" as P
database  "Payments-DB" as D
participant "Notification-svc" as N

C  -> P : POST /refund
P  -> D : update txn status
P <- D : success
P  -> N : emit REFUND_SUCCESS
@enduml
```

```python
# utils/refund.py
def refund(txn_id: str) -> None:
    """Trigger a refund and log outcome."""
    ...
```
