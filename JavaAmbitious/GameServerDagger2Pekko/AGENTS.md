# AGENTS.md - Game Server Development Guide

This document provides essential information for agents working with this Dagger2 + Pekko game server codebase.

## Related Documentation

This project includes several complementary documentation files:

- **README.md** - Detailed technical explanation in Chinese covering architecture decisions and best practices
- **QUICKSTART.md** - 5-minute quick start guide for new developers
- **THREAD_SAFETY_GUIDE.md** - Deep dive into thread safety concepts and implementation details
- **DAGGER2_SCALING_GUIDE.md** - Guidelines for extending Dagger2 modules as the project grows

## Project Overview

This is a **Java-based game server** that combines:
- **Pekko Actor Framework** for concurrent message processing
- **Dagger2** for compile-time dependency injection
- **PostgreSQL** with Hibernate ORM for data persistence
- **Maven** as the build system

**Technology Stack:**
- **Java 11** (Maven compiler target)
- **Pekko 1.0.2** (Actor framework with cluster support)
- **Dagger2 2.51** (Compile-time dependency injection)
- **PostgreSQL 42.6.0** (Database)
- **Hibernate 6.2.0** (ORM)
- **HikariCP 5.0.1** (Connection pooling)
- **SLF4J 2.0.11 + Logback 1.4.14** (Logging)
- **Guava 33.0.0** (Google libraries for utilities)

### Core Architecture

```
GameServerComponent (Dagger2 DI Container)
├── InfrastructureModule (Infrastructure)
│   ├── GameServerModule (Configuration)
│   ├── ActorSystemModule (Pekko Actors)
│   └── DataAccessModule (Database/DAOs)
└── BusinessModule (Business Logic)
    └── ServiceModule (Game Services)

├── Actor Layer (Message-Driven Processing)
│   ├── PlayerActor (Per-player state management)
│   ├── MapActor (Map/zone management)
│   └── Actor Factories (Dependency-injected creation)
└── Data Layer (Persistence)
    ├── DAOs (Data Access Objects)
    └── Entities (Database models)
```

## Essential Commands

### Build and Run
```bash
# Clean compile (removes all build artifacts and compiles)
mvn clean compile

# Clean and quick rebuild (removes all build artifacts and compiles)
./rebuild.sh

# Run the demo server
mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap"

# Build executable JAR
mvn clean package

# Run from JAR
java -jar target/dagger2-pekko-gameserver-1.0.0.jar
```

### Testing
```bash
# Run all tests
mvn test

# Run specific test class
mvn test -Dtest=GameServerComponentTest

# Run with debug output
mvn test -Dmaven.test.debug=true

# Run test with specific Pekko log level
mvn test -Dpekko.loglevel=DEBUG
```

### Development
```bash
# Check for compilation errors
mvn validate

# Generate dependency tree
mvn dependency:tree

# Find Dagger2 generated classes
find target -name "Dagger*.java"

# Check for dependency conflicts
mvn dependency:tree | grep "omitted for conflict"

# Build without running tests (faster compilation)
mvn clean compile -DskipTests

# Build with offline mode (useful for disconnected development)
mvn clean compile -o
```

## Project Structure

### Key Directories
- `src/main/java/com/gameserver/` - Main application code
  - `di/` - Dagger2 dependency injection modules
  - `actor/` - Pekko actor implementations
  - `service/` - Business logic services
  - `dao/` - Data access layer
  - `entity/` - Database entities
  - `config/` - Configuration classes
  - `model/` - Domain models
- `src/main/resources/` - Configuration files
  - `application.properties` - Server configuration
  - `hibernate.xml` - Database configuration (Hibernate ORM settings)

### Module Organization
- **InfrastructureModule**: Core infrastructure (ActorSystem, config, database)
- **BusinessModule**: Business logic services
- **Actor Factories**: Dependency-injected actor creation
- **DAOs**: Database access with connection pooling

### Naming Conventions
- **Actor classes**: `*Actor` (e.g., `PlayerActor`, `MapActor`)
- **Actor factories**: `*ActorFactory` (e.g., `PlayerActorFactory`)
- **Messages**: `*Message` with static inner classes (e.g., `PlayerMessage.Login`)
- **Services**: Interface + `Impl` suffix (e.g., `SkillService`, `SkillServiceImpl`)
- **DAOs**: Interface + `Impl` suffix (e.g., `PlayerDAO`, `PlayerDAOImpl`)
- **Components**: `*Component` interface (e.g., `GameServerComponent`)
- **Modules**: `*Module` (e.g., `GameServerModule`, `ServiceModule`)

### Source Code Organization
```
com.gameserver/
├── GameServerBootstrap.java          # Application entry point
├── config/
│   └── GameServerConfig.java         # Configuration management
├── di/
│   ├── GameServerComponent.java      # Main Dagger2 component
│   ├── InfrastructureModule.java     # Infrastructure dependencies
│   ├── BusinessModule.java           # Business logic dependencies
│   └── ... (specific modules)
├── actor/
│   ├── PlayerActor.java             # Player state management
│   ├── PlayerActorFactory.java      # Player actor creation
│   ├── PlayerMessage.java           # Player-related messages
│   ├── MapActor.java                # Map/zone management
│   ├── MapActorFactory.java         # Map actor creation
│   └── MapMessage.java              # Map-related messages
├── service/
│   ├── SkillService.java             # Skill system interface
│   ├── MapService.java               # Map system interface
│   └── impl/
│       ├── SkillServiceImpl.java    # Skill system implementation
│       └── MapServiceImpl.java       # Map system implementation
├── dao/
│   ├── PlayerDAO.java                # Player data access interface
│   ├── BackpackDAO.java              # Backpack data access interface
│   └── impl/
│       ├── PlayerDAOImpl.java       # Player data access implementation
│       └── BackpackDAOImpl.java     # Backpack data access implementation
├── entity/
│   ├── Player.java                   # Player entity
│   └── Backpack.java                 # Backpack entity
└── model/
    └── Player.java                   # Player domain model
```

## Code Patterns and Conventions

### 1. Dagger2 Dependency Injection

**Component Structure:**
```java
@Singleton
@Component(modules = {
    InfrastructureModule.class,
    BusinessModule.class
})
public interface GameServerComponent {
    // Core dependencies
    ActorSystem getActorSystem();
    GameServerConfig getGameServerConfig();
    
    // Data access
    PlayerDAO getPlayerDAO();
    
    // Services
    SkillService getSkillService();
    MapService getMapService();
    
    // Actor factories (key entry points)
    PlayerActorFactory getPlayerActorFactory();
    MapActorFactory getMapActorFactory();
}
```

**Module Pattern:**
```java
@Module
public class GameServerModule {
    @Singleton
    @Provides
    static GameServerConfig provideGameServerConfig() {
        // Load from properties, handle fallbacks
    }
}
```

### 2. Actor Implementation Pattern

**Actor Structure:**
```java
public class PlayerActor extends AbstractActor {
    private final long playerId;
    private final PlayerDAO playerDAO;
    private final SkillService skillService;
    
    // Constructor injection via Dagger2
    public PlayerActor(long playerId, PlayerDAO playerDAO, SkillService skillService) {
        // Initialize dependencies
    }
    
    @Override
    public Receive createReceive() {
        return receiveBuilder()
            .match(PlayerMessage.Login.class, this::handleLogin)
            .match(PlayerMessage.Attack.class, this::handleAttack)
            .build();
    }
}
```

**Actor Factory Pattern:**
```java
public class PlayerActorFactory {
    private final ActorSystem actorSystem;
    private final PlayerDAO playerDAO;
    private final SkillService skillService;
    
    @Inject
    public PlayerActorFactory(ActorSystem actorSystem, PlayerDAO playerDAO, SkillService skillService) {
        // Dagger2 injects dependencies
    }
    
    public ActorRef create(long playerId) {
        Props props = Props.create(PlayerActor.class, playerId, playerDAO, skillService);
        return actorSystem.actorOf(props, "player-" + playerId);
    }
}
```

### 3. Message Pattern

**Immutable Messages:**
```java
public static final class PlayerMessage {
    public final long playerId;
    public final String action;
    
    public PlayerMessage(long playerId, String action) {
        this.playerId = playerId;
        this.action = action;
    }
}
```

### 4. Service Layer Pattern

**Interface-Implementation:**
```java
public interface SkillService {
    int calculateDamage(long attackerId, int skillId, long targetId);
}

@Singleton
public class SkillServiceImpl implements SkillService {
    @Override
    public int calculateDamage(long attackerId, int skillId, long targetId) {
        // Business logic implementation
    }
}
```

## Thread Safety Guidelines

### Core Principles
1. **Dagger2 ensures thread safety at compile time** - all dependencies are singletons
2. **Pekko Actors are single-threaded** - message processing is inherently thread-safe
3. **Immutable messages** - prevent concurrent modification issues
4. **Database operations use blocking-dispatcher** - avoid blocking actor threads

### Key Thread Safety Rules

- **Never share mutable state between actors** - use message passing instead
- **Use blocking-dispatcher for database operations**:
  ```java
  CompletableFuture.supplyAsync(
      () -> playerDAO.findById(playerId),
      context().system().dispatchers().lookup("blocking-dispatcher")
  ).thenAccept(player -> {
      self().tell(new LoginResult(player), self());
  });
  ```
- **All @Singleton dependencies must be thread-safe** (HikariCP, immutable services)
- **Actor messages must be immutable** - use `final` fields or Java records

### Common Pitfalls to Avoid

1. **Blocking Actor Threads** - Never perform database I/O in actor message handlers without async patterns
2. **Shared Mutable State** - Avoid static variables or shared collections between actors
3. **Circular Dependencies** - Dagger2 will catch these at compile time
4. **Direct Field Access** - Never access actor fields from outside the actor

## Configuration

### Application Properties (`src/main/resources/application.properties`)
```properties
# Server Configuration
server.port=8888
server.host=0.0.0.0
actor.system.name=GameServer
actor.dispatcher=default

# Database Configuration
db.url=jdbc:postgresql://192.168.199.199:5432/gamedb
db.user=postgres
db.password=hqy123
db.pool.size=20
```

### Environment Variable Override
The system supports environment variable overrides. Set:
- `SERVER_PORT` - Override server port
- `DB_URL` - Override database URL
- `DB_USER` - Override database username
- `DB_PASSWORD` - Override database password

## Database Configuration

### Hibernate Setup
- Uses `hibernate.xml` for ORM configuration
- PostgreSQL with JSONB support via `hibernate-types`
- HikariCP connection pooling (20 connections by default)
- Entities in `src/main/java/com/gameserver/entity/`

### Entity Pattern
```java
@Entity
@Table(name = "players")
public class Player {
    @Id
    private long playerId;
    
    @Column(name = "account_name")
    private String account;
    
    // Additional fields, getters, setters
}
```

## Testing Strategy

### Unit Tests (Framework Available)
- Test structure is available in the codebase but may not be implemented yet
- Location: `src/test/java/`
- Expected to use Pekko TestKit for actor testing
- Mockito for dependency mocking

### Test Structure (Template Available)
```java
@Test
public void testPlayerActorLogin() {
    new TestKit(system) {
        {
            // Create mock dependencies
            PlayerDAO playerDAO = mock(PlayerDAO.class);
            when(playerDAO.findById(1001))
                .thenReturn(Optional.of(mockPlayer));
            
            // Create actor
            ActorRef playerActor = system.actorOf(
                Props.create(PlayerActor.class, 1001L, playerDAO, skillService),
                "test-player"
            );
            
            // Send message and verify response
            playerActor.tell(new PlayerMessage.Login(...), getRef());
            expectMsgClass(PlayerMessage.LoginResponse.class);
        }
    };
}
```

### Running Tests
```bash
# Run all tests (if implemented)
mvn test

# Run with coverage (JaCoCo plugin available in pom.xml)
mvn clean test jacoco:report
```

## Debugging and Troubleshooting

### Common Issues

1. **Dagger2 Component Not Found**
   - Ensure `pom.xml` has Dagger2 annotation processor configured
   - Run `mvn clean compile` to generate Dagger code

2. **Actor Creation Fails**
   - Check if all dependencies are provided in modules
   - Verify ActorSystem is properly initialized

3. **Database Connection Issues**
   - Check database connectivity
   - Verify HikariCP pool configuration
   - Review `hibernate.xml` settings

### Debug Commands
```bash
# Enable verbose Pekko logging
mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap" \
    -Dpekko.loglevel=DEBUG

# Find generated Dagger2 classes
find target -name "Dagger*.java"

# Check dependency tree
mvn dependency:tree
```

## Performance Considerations

### Memory Management
- **Singletons** - Dagger2 ensures one instance per dependency
- **Actor Lifecycle** - Actors are garbage collected when no longer referenced
- **Connection Pooling** - HikariCP reuses database connections

### Concurrency
- **Horizontal Scaling** - Use Pekko Cluster for distributed deployment
- **Actor Parallelism** - Each Actor processes messages sequentially
- **I/O Operations** - Always use async patterns for blocking operations

## Development Workflow

1. **Setup Environment**
   ```bash
   git clone <repository>
   cd GameServerDagger2Pekko
   mvn clean compile
   ```

2. **Run Demo**
   ```bash
   mvn exec:java -Dexec.mainClass="com.gameserver.GameServerBootstrap"
   ```

3. **Add New Features**
   - Create new services in `service/` directory
   - Add corresponding modules in `di/` directory
   - Create actors for new game entities
   - Update Dagger2 component if needed

4. **Testing**
   ```bash
   mvn test
   ```

## Best Practices

1. **Follow the existing module structure** - new dependencies should go into appropriate modules
2. **Use immutable messages** - final fields or Java records
3. **Keep actors focused** - one responsibility per actor
4. **Leverage Dagger2 singletons** - avoid manual singleton management
5. **Use async patterns** - never block actor threads
6. **Document thread safety** - comment on thread safety assumptions
7. **Follow naming conventions** - consistent with existing code

## Learning Resources

- **[Pekko Documentation](https://pekko.apache.org/)**
- **[Dagger2 Documentation](https://dagger.dev/)**
- **[Hibernate ORM Documentation](https://docs.jboss.org/hibernate/orm/6.2/)**
- **[HikariCP Documentation](https://github.com/brettwooldridge/HikariCP)**