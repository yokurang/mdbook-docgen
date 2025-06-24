# 01 – Service Overview

Lup surm dolor sit amet, consectetur adipiscing elit.  
Praesent tincidunt **finibus** velit, non aliquet urna gravida a.  
Duis vitae eros sed augue sagittis lacinia.

```mermaid
classDiagram
    direction TB
    class User {
        +uuid id
        +string email
        +bool   verified
    }
    class Account {
        +uuid id
        +decimal balance
    }
    User "1" --> "1..*" Account : owns
````

```plantuml
@startuml
skinparam componentStyle rectangle
component "payments-service" as S
database  "payments-db"      as DB
queue     "event-bus"        as Q

S   --> DB : JDBC read/write
S   ->  Q  : publish events
@enduml
```

```rust
// src/main.rs
fn main() {
    println!("Hello, payments-service!");
}
```

