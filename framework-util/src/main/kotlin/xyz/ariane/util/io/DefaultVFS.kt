package xyz.ariane.util.io

import com.google.common.base.Preconditions
import org.slf4j.LoggerFactory
import java.io.*
import java.net.MalformedURLException
import java.net.URL
import java.net.URLEncoder
import java.util.*
import java.util.jar.JarInputStream

/**
 * A default implementation of [VFS] that works for most application servers.
 *
 * @author Ben Gunter
 */
class DefaultVFS : VFS() {

    override val isValid: Boolean = true

    @Throws(IOException::class)
    public override fun list(url: URL, path: String): List<String> {
        var inputStream: InputStream? = null
        try {
            var resources: MutableList<String> = ArrayList()

            // First, try to find the URL of a JAR file containing the requested resource. If a JAR
            // file is found, then we'll list child resources by reading the JAR.
            val jarUrl = findJarForResource(url)
            if (jarUrl != null) {
                inputStream = jarUrl.openStream()
                if (log.isDebugEnabled) {
                    log.debug("Listing $url")
                }
                resources = listResources(JarInputStream(inputStream), path)
            } else {
                var children: MutableList<String> = ArrayList()
                try {
                    if (isJar(url)) {
                        // Some versions of JBoss VFS might give a JAR stream even if the resource
                        // referenced by the URL isn't actually a JAR
                        inputStream = url.openStream()
                        val jarInput = JarInputStream(inputStream)
                        if (log.isDebugEnabled) {
                            log.debug("Listing $url")
                        }
                        var entry = jarInput.nextJarEntry
                        while (entry != null) {
                            if (log.isDebugEnabled) {
                                log.debug("Jar entry: " + entry.name)
                            }
                            children.add(entry.name)

                            entry = jarInput.nextJarEntry
                        }
                        jarInput.close()

                    } else {
                        /*
                         * Some servlet containers allow reading from directory resources like a
                         * text file, listing the child resources one per line. However, there is no
                         * way to differentiate between directory and file resources just by reading
                         * them. To work around that, as each line is read, try to look it up via
                         * the class loader as a child of the current resource. If any line fails
                         * then we assume the current resource is not a directory.
                         */
                        inputStream = url.openStream()
                        val reader = BufferedReader(InputStreamReader(inputStream))
                        val lines = ArrayList<String>()
                        var line = reader.readLine()
                        while (line != null) {
                            if (log.isDebugEnabled) {
                                log.debug("Reader entry: $line")
                            }

                            lines.add(line)
                            if (VFS.getResources("$path/$line").isEmpty()) {
                                lines.clear()
                                break
                            }

                            line = reader.readLine()
                        }

                        if (!lines.isEmpty()) {
                            if (log.isDebugEnabled) {
                                log.debug("Listing $url")
                            }
                            children.addAll(lines)
                        }
                    }
                } catch (e: FileNotFoundException) {
                    /*
                     * For file URLs the openStream() call might fail, depending on the servlet
                     * container, because directories can't be opened for reading. If that happens,
                     * then list the directory directly instead.
                     */
                    if ("file" == url.protocol) {
                        val file = File(url.file)
                        if (log.isDebugEnabled) {
                            log.debug("Listing directory " + file.absolutePath)
                        }
                        if (file.isDirectory) {
                            if (log.isDebugEnabled) {
                                log.debug("Listing $url")
                            }
                            val list = file.list()
                            children = Arrays.asList(*Preconditions.checkNotNull(list))
                        }
                    } else {
                        // No idea where the exception came from so rethrow it
                        throw e
                    }
                }

                // The URL prefix to use when recursively listing child resources
                var prefix = url.toExternalForm()
                if (!prefix.endsWith("/")) {
                    prefix = "$prefix/"
                }

                // Iterate over immediate children, adding files and recursing into directories
                for (child in children) {
                    val resourcePath = "$path/$child"
                    resources.add(resourcePath)
                    val childUrl = URL(prefix + child)
                    resources.addAll(list(childUrl, resourcePath))
                }
            }

            return resources
        } finally {
            if (inputStream != null) {
                try {
                    inputStream.close()
                } catch (e: Exception) {
                    // Ignore
                }

            }
        }
    }

    /**
     * List the names of the entries in the given [JarInputStream] that begin with the
     * specified `path`. Entries will match with or without a leading slash.
     *
     * @param jar  The JAR input stream
     * @param p The leading path to match
     * @return The names of all the matching entries
     * @throws IOException If I/O errors occur
     */
    @Throws(IOException::class)
    private fun listResources(jar: JarInputStream, p: String): MutableList<String> {
        var path = p
        // Include the leading and trailing slash when matching names
        if (!path.startsWith("/")) {
            path = "/$path"
        }
        if (!path.endsWith("/")) {
            path = "$path/"
        }

        // Iterate over the entries and collect those that begin with the requested path
        val resources = ArrayList<String>()
        var entry = jar.nextJarEntry
        while (entry != null) {
            if (!entry.isDirectory) {
                // Add leading slash if it's missing
                var name = entry.name
                if (!name.startsWith("/")) {
                    name = "/$name"
                }

                // Check file name
                if (name.startsWith(path)) {
                    if (log.isDebugEnabled) {
                        log.debug("Found resource: $name")
                    }

                    // Trim leading slash
                    resources.add(name.substring(1))
                }
            }

            entry = jar.nextJarEntry
        }
        return resources
    }

    /**
     * Attempts to deconstruct the given URL to find a JAR file containing the resource referenced
     * by the URL. That is, assuming the URL references a JAR entry, this method will return a URL
     * that references the JAR file containing the entry. If the JAR cannot be located, then this
     * method returns null.
     *
     * @param u The URL of the JAR entry.
     * @return The URL of the JAR file, if one is found. Null if not.
     */
    private fun findJarForResource(u: URL): URL? {
        var url = u
        if (log.isDebugEnabled) {
            log.debug("Find JAR URL: $url")
        }

        // If the file part of the URL is itself a URL, then that URL probably points to the JAR
        try {

            while (true) {
                url = URL(url.file)
                if (log.isDebugEnabled) {
                    log.debug("Inner URL: $url")
                }
            }
        } catch (e: MalformedURLException) {
            // This will happen at some point and serves as a break in the loop
        }

        // Look for the .jar extension and chop off everything after that
        val jarUrl = StringBuilder(url.toExternalForm())
        val index = jarUrl.lastIndexOf(".jar")
        if (index >= 0) {
            jarUrl.setLength(index + 4)
            if (log.isDebugEnabled) {
                log.debug("Extracted JAR URL: $jarUrl")
            }
        } else {
            if (log.isDebugEnabled) {
                log.debug("Not a JAR: $jarUrl")
            }
            return null
        }

        // Try to open and test it
        try {
            var testUrl = URL(jarUrl.toString())
            if (isJar(testUrl)) {
                return testUrl
            } else {
                // WebLogic fix: check if the URL's file exists in the filesystem.
                if (log.isDebugEnabled) {
                    log.debug("Not a JAR: $jarUrl")
                }
                jarUrl.replace(0, jarUrl.length, testUrl.file)
                var file = File(jarUrl.toString())

                // File name might be URL-encoded
                if (!file.exists()) {
                    try {
                        file = File(URLEncoder.encode(jarUrl.toString(), "UTF-8"))
                    } catch (e: UnsupportedEncodingException) {
                        throw RuntimeException("Unsupported encoding?  UTF-8?  That's unpossible.")
                    }

                }

                if (file.exists()) {
                    if (log.isDebugEnabled) {
                        log.debug("Trying real file: " + file.absolutePath)
                    }
                    testUrl = file.toURI().toURL()
                    if (isJar(testUrl)) {
                        return testUrl
                    }
                }
            }
        } catch (e: MalformedURLException) {
            log.warn("Invalid JAR URL: $jarUrl")
        }

        if (log.isDebugEnabled) {
            log.debug("Not a JAR: $jarUrl")
        }
        return null
    }

    /**
     * Returns true if the resource located at the given URL is a JAR file.
     *
     * @param url    The URL of the resource to test.
     * @param buffer A buffer into which the first few bytes of the resource are read. The buffer
     * must be at least the size of [.JAR_MAGIC]. (The same buffer may be reused
     * for multiple calls as an optimization.)
     */
    private fun isJar(url: URL, buffer: ByteArray = ByteArray(JAR_MAGIC.size)): Boolean {
        var inputStream: InputStream? = null
        try {
            inputStream = url.openStream()
            inputStream.read(buffer, 0, JAR_MAGIC.size)
            if (Arrays.equals(buffer, JAR_MAGIC)) {
                if (log.isDebugEnabled) {
                    log.debug("Found JAR: $url")
                }
                return true
            }
        } catch (e: Exception) {
            // Failure to read the stream means this is not a JAR
        } finally {
            if (inputStream != null) {
                try {
                    inputStream.close()
                } catch (e: Exception) {
                    // Ignore
                }

            }
        }

        return false
    }

    companion object {

        private val log = LoggerFactory.getLogger(ClassResolver::class.java)

        /**
         * The magic header that indicates a JAR (ZIP) file.
         */
        private val JAR_MAGIC = byteArrayOf('P'.code.toByte(), 'K'.code.toByte(), 3, 4)
    }
}
