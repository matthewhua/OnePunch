import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    idea
    `java-library`
    java
    kotlin("jvm") version "1.6.21"
    `maven-publish`
    id("org.jetbrains.kotlin.plugin.noarg") version "1.6.21"
    id("org.jetbrains.kotlin.plugin.allopen") version "1.6.21"
}

group = "org.matt"
version = "4.0.16"

repositories {
    mavenCentral()
}

// 这里算是定义外部依赖
val jacksonVersion = "2.12.5"
val jacksonDataBindVersion = "2.12.5"

dependencies {
    testImplementation(kotlin("test"))
    api("org.apache.poi:poi:3.17") {
        exclude(group = "log4j")
    }

    api("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.6.0")
    api("org.jetbrains.kotlin:kotlin-script-runtime:1.6.21")
    api("redis.clients:jedis:4.3.1")

    api("com.typesafe.akka:akka-actor_2.13:2.7.0")
    api("com.typesafe.akka:akka-http_2.13:10.4.0")
    api("com.typesafe.akka:akka-cluster-metrics_2.13:2.7.0")
    api("com.typesafe.akka:akka-cluster_2.13:2.7.0")
    api("com.typesafe.akka:akka-cluster-tools_2.13:2.7.0")
    api("com.typesafe.akka:akka-cluster-sharding_2.13:2.7.0")
    api("com.typesafe.akka:akka-http-jackson_2.13:10.5.1")


    api("com.lightbend.akka.management:akka-management-cluster-bootstrap_2.13:1.2.0")
    api("io.grpc:grpc-protobuf:1.53.0")
    api("com.alibaba:druid:1.2.16")
    api("org.apache.logging.log4j:log4j-to-slf4j:2.11.1")
    api("ch.qos.logback:logback-classic:1.2.10")
    // jackson
    api("com.fasterxml.jackson.core:jackson-core:${jacksonVersion}")
    api("com.fasterxml.jackson.core:jackson-databind:${jacksonDataBindVersion}")
    api("com.fasterxml.jackson.core:jackson-annotations:${jacksonVersion}")
    api("com.fasterxml.jackson.dataformat:jackson-dataformat-xml:${jacksonVersion}")
    api("com.fasterxml.jackson.dataformat:jackson-dataformat-yaml:${jacksonVersion}")
    api("com.fasterxml.jackson.module:jackson-module-kotlin:${jacksonVersion}")

    api("com.alibaba:druid:1.2.16")
    api("com.github.ben-manes.caffeine:guava:3.1.7")
    //api("org.apache.shardingsphere:shardingsphere-jdbc-core:5.2.1")
    api("io.prometheus:simpleclient_hotspot:0.16.0")
    api("io.prometheus:simpleclient_httpserver:0.16.0")
    api("org.apache.curator:curator-client:5.4.0")
    api("org.apache.curator:curator-framework:5.4.0")
    api("org.apache.curator:curator-recipes:5.4.0")
    // lz4
    api("org.lz4:lz4-java:1.8.0")
    api("org.hibernate:hibernate-core:5.6.6.Final")

    api("com.esotericsoftware:kryo:5.4.0")
    api("org.reflections:reflections:0.9.12")

    api("org.dom4j:dom4j:2.1.3")

    api("org.apache.kafka:kafka-clients:3.5.1")
    api("org.jetbrains.kotlinx:kotlinx-coroutines-jdk8:1.5.2")
    api(kotlin("stdlib"))

    api("io.netty:netty-codec-http:4.1.97.Final")

}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<KotlinCompile> {
    kotlinOptions {
        apiVersion = "1.6"
        languageVersion = "1.6"

        // Generate metadata for Java 1.8 reflection on method parameters
        javaParameters = true

        // Target version of the generated JVM bytecode (1.6 or 1.8), default is 1.6
        jvmTarget = "11"
    }
}

tasks.register<Jar>("ktSourcesJar") {
    archiveClassifier.set("kt-sources")
    from(sourceSets["main"].allSource)
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            groupId = "xyz.matt"
            artifactId = "framework-util"
            version = "4.0.16"
            from(components["java"])
            artifact(tasks["ktSourcesJar"])
        }
    }

    repositories {
        mavenLocal()
    }
}
