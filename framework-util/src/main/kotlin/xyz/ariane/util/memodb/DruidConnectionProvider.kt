package xyz.ariane.util.memodb

import com.alibaba.druid.pool.DruidDataSource
import com.alibaba.druid.pool.DruidDataSourceFactory
import org.hibernate.engine.jdbc.connections.spi.ConnectionProvider
import org.hibernate.service.UnknownUnwrapTypeException
import org.hibernate.service.spi.Configurable
import org.hibernate.service.spi.Stoppable
import java.sql.Connection
import javax.sql.DataSource

/**
 * [ConnectionProvider] implementation by [DruidDataSource]
 *
 */
class DruidConnectionProvider : ConnectionProvider, Configurable, Stoppable {

    val dataSource: DruidDataSource = DruidDataSource()

    override fun stop() {
        dataSource.close()
    }

    override fun configure(props: MutableMap<Any?, Any?>?) {
        DruidDataSourceFactory.config(dataSource, props)
    }

    override fun isUnwrappableAs(unwrapType: Class<*>): Boolean {
        return ConnectionProvider::class.java == unwrapType ||
                DruidConnectionProvider::class.java.isAssignableFrom(unwrapType) ||
                DataSource::class.java.isAssignableFrom(unwrapType)
    }

    override fun <T> unwrap(unwrapType: Class<T>): T {
        return if (ConnectionProvider::class.java == unwrapType ||
            DruidConnectionProvider::class.java.isAssignableFrom(unwrapType)
        ) {
            this as T
        } else if (DataSource::class.java.isAssignableFrom(unwrapType)) {
            dataSource as T
        } else {
            throw UnknownUnwrapTypeException(unwrapType)
        }
    }

    override fun supportsAggressiveRelease(): Boolean = false

    override fun getConnection(): Connection? {
        return dataSource.connection
    }

    override fun closeConnection(conn: Connection) {
        conn.close()
    }
}