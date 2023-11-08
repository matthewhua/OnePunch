package xyz.ariane.util.io

import org.slf4j.LoggerFactory
import java.io.IOException
import java.net.URL
import java.util.*

/**
 * Provides a very simple API for accessing resources within an application server.
 *
 * @author Ben Gunter
 */
abstract class VFS {

    /**
     * Return true if the [VFS] implementation is valid for the current environment.
     */
    abstract val isValid: Boolean

    /**
     * Recursively list the full resource path of all the resources that are children of the
     * resource identified by a URL.
     *
     * @param url     The URL that identifies the resource to list.
     * @param forPath The path to the resource that is identified by the URL. Generally, this is the
     * value passed to [.getResources] to get the resource URL.
     * @return A list containing the names of the child resources.
     * @throws IOException If I/O errors occur
     */
    @Throws(IOException::class)
    protected abstract fun list(url: URL, forPath: String): List<String>

    /**
     * Recursively list the full resource path of all the resources that are children of all the
     * resources found at the specified path.
     *
     * @param path The path of the resource(s) to list.
     * @return A list containing the names of the child resources.
     * @throws IOException If I/O errors occur
     */
    @Throws(IOException::class)
    fun list(path: String): List<String> {
        val names = ArrayList<String>()
        for (url in getResources(path)) {
            names.addAll(list(url, path))
        }
        return names
    }

    companion object {

        private val log = LoggerFactory.getLogger(ClassResolver::class.java)

        /**
         * The built-in implementations.
         */
        val IMPLEMENTATIONS = arrayOf<Class<*>>(DefaultVFS::class.java)

        /**
         * Singleton instance.
         */
        private var instance: VFS? = null

        /**
         * Get the singleton [VFS] instance. If no [VFS] implementation can be found for the
         * current environment, then this method returns null.
         */
        fun getInstance(): VFS {
            val inst = instance
            if (inst != null) {
                return inst
            }

            // Try the user implementations first, then the built-ins
            val impls = ArrayList<Class<out VFS>>()
            impls.addAll(Arrays.asList(*IMPLEMENTATIONS as Array<Class<out VFS>>))

            // Try each implementation class until a valid one is found
            var vfs: VFS? = null
            var i = 0
            while (vfs == null || !vfs.isValid) {
                val impl = impls[i]
                try {
                    vfs = impl.getDeclaredConstructor().newInstance()
                    if (vfs == null || !vfs.isValid) {
                        if (log.isDebugEnabled) {
                            log.debug("VFS implementation " + impl.name +
                                    " is not valid in this environment.")
                        }
                    }
                } catch (e: InstantiationException) {
                    log.error("Failed to instantiate $impl", e)
                    throw RuntimeException(e)
                } catch (e: IllegalAccessException) {
                    log.error("Failed to instantiate $impl", e)
                    throw RuntimeException(e)
                }

                i++
            }

            if (log.isDebugEnabled) {
                log.debug("Using VFS adapter " + vfs.javaClass.name)
            }

            VFS.instance = vfs
            return vfs
        }

        /**
         * Get a class by name. If the class is not found then return null.
         */
        fun getClass(className: String): Class<*>? {
            try {
                return Thread.currentThread().contextClassLoader.loadClass(className)
            } catch (e: ClassNotFoundException) {
                if (log.isDebugEnabled) {
                    log.debug("Class not found: $className")
                }
                return null
            }

        }

        /**
         * Get a list of [URL]s from the context classloader for all the resources found at the
         * specified path.
         *
         * @param path The resource path.
         * @return A list of [URL]s, as returned by [ClassLoader.getResources].
         * @throws IOException If I/O errors occur
         */
        @Throws(IOException::class)
        fun getResources(path: String): List<URL> {
            return Collections.list(Thread.currentThread().contextClassLoader.getResources(path))
        }
    }
}
