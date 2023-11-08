package xyz.ariane.util.io

import org.slf4j.LoggerFactory
import xyz.ariane.util.io.ClassResolver.Test
import java.io.IOException
import java.util.*

/**
 *
 * ClassResolver is used to locate classes that are available in the/a class path and meet
 * arbitrary conditions. The two most common conditions are that a class implements/extends
 * another class, or that is it annotated with a specific annotation. However, through the use
 * of the [Test] class it is possible to search using arbitrary conditions.
 *
 *
 *
 * A ClassLoader is used to locate all locations (directories and jar files) in the class
 * path that contain classes within certain packages, and then to load those classes and
 * check them. By default the ClassLoader returned by
 * `Thread.currentThread().getContextClassLoader()` is used, but this can be overridden
 * by calling [.setClassLoader] prior to invoking any of the `find()`
 * methods.
 *
 *
 *
 * General searches are initiated by calling the
 * [.find] ()} method and supplying
 * a package name and a Test instance. This will cause the named package **and all sub-packages**
 * to be scanned for classes that meet the test. There are also utility methods for the common
 * use cases of scanning multiple packages for extensions of particular classes, or classes
 * annotated with a specific annotation.
 *
 *
 *
 * The standard usage pattern for the ClassResolver class is as follows:
 *
 *
 * <pre>
 * ClassResolver&lt;ActionBean&gt; resolver = new ClassResolver&lt;ActionBean&gt;();
 * resolver.findImplementation(ActionBean.class, pkg1, pkg2);
 * resolver.find(new CustomTest(), pkg1);
 * resolver.find(new CustomTest(), pkg2);
 * Collection&lt;ActionBean&gt; beans = resolver.getClasses();
</pre> *
 *
 * @author Tim Fennell
 */
class ClassResolver<T> {

    /**
     * The set of matches being accumulated.
     */
    private val matches = HashSet<Class<out T>>()

    /**
     * The ClassLoader to use when looking for classes. If null then the ClassLoader returned
     * by Thread.currentThread().getContextClassLoader() will be used.
     */
    /**
     * Returns the classloader that will be used for scanning for classes. If no explicit
     * ClassLoader has been set by the calling, the context class loader will be used.
     *
     * @return the ClassLoader that will be used to scan for classes
     */
    /**
     * Sets an explicit ClassLoader that should be used when scanning for classes. If none
     * is set then the context classloader will be used.
     *
     * @param classloader a ClassLoader to use when scanning for classes
     */
    var classLoader: ClassLoader? = null
        get() = if (field == null) Thread.currentThread().contextClassLoader else field

    /**
     * Provides access to the classes discovered so far. If no calls have been made to
     * any of the `find()` methods, this set will be empty.
     *
     * @return the set of classes that have been discovered.
     */
    val classes: Set<Class<out T>>
        get() = matches

    /**
     * A simple interface that specifies how to test classes to determine if they
     * are to be included in the results produced by the ClassResolver.
     */
    interface Test {

        /**
         * Will be called repeatedly with candidate classes. Must return True if a class
         * is to be included in the results, false otherwise.
         */
        fun matches(type: Class<*>): Boolean
    }

    /**
     * A Test that checks to see if each class is assignable to the provided class. Note
     * that this test will match the parent type itself if it is presented for matching.
     */
    class IsA(private val parent: Class<*>) : Test {

        /**
         * Returns true if type is assignable to the parent type supplied in the constructor.
         */
        override fun matches(type: Class<*>): Boolean {
            return parent.isAssignableFrom(type)
        }

        override fun toString(): String {
            return "is assignable to " + parent.simpleName
        }
    }

    /**
     * A Test that checks to see if each class is annotated with a specific annotation. If it
     * is, then the test returns true, otherwise false.
     */
    class AnnotatedWith(private val annotation: Class<out Annotation>) : Test {

        /**
         * Returns true if the type is annotated with the class provided to the constructor.
         */
        override fun matches(type: Class<*>): Boolean {
            return type.isAnnotationPresent(annotation)
        }

        override fun toString(): String {
            return "annotated with @" + annotation.simpleName
        }
    }

    /**
     * Attempts to discover classes that are assignable to the type provided. In the case
     * that an interface is provided this method will collect implementations. In the case
     * of a non-interface class, subclasses will be collected.  Accumulated classes can be
     * accessed by calling [.getClasses].
     *
     * @param parent       the class of interface to find subclasses or implementations of
     * @param packageNames one or more package names to scan (including subpackages) for classes
     */
    fun findImplementations(parent: Class<*>, vararg packageNames: String): ClassResolver<T> {
        if (packageNames == null) {
            return this
        }

        val test = IsA(parent)
        for (pkg in packageNames) {
            find(test, pkg)
        }

        return this
    }

    /**
     * Attempts to discover classes that are annotated with the annotation. Accumulated
     * classes can be accessed by calling [.getClasses].
     *
     * @param annotation   the annotation that should be present on matching classes
     * @param packageNames one or more package names to scan (including subpackages) for classes
     */
    fun findAnnotated(annotation: Class<out Annotation>, vararg packageNames: String): ClassResolver<T> {
        if (packageNames == null) {
            return this
        }

        val test = AnnotatedWith(annotation)
        for (pkg in packageNames) {
            find(test, pkg)
        }

        return this
    }

    /**
     * Scans for classes starting at the package provided and descending into subpackages.
     * Each class is offered up to the Test as it is discovered, and if the Test returns
     * true the class is retained.  Accumulated classes can be fetched by calling
     * [.getClasses].
     *
     * @param test        an instance of [Test] that will be used to filter classes
     * @param packageName the name of the package from which to start scanning for
     * classes, e.g. `net.sourceforge.stripes`
     */
    fun find(test: Test, packageName: String) {
        val path = getPackagePath(packageName)

        try {
            val children = VFS.getInstance().list(path)
            children.stream().filter { child -> child.endsWith(".class") }.forEach { child -> addIfMatching(test, child) }
        } catch (ioe: IOException) {
            log.error("Could not read package: $packageName", ioe)
        }

    }

    /**
     * Converts a Java package name to a path that can be looked up with a call to
     * [ClassLoader.getResources].
     *
     * @param packageName The Java package name to convert to a path
     */
    protected fun getPackagePath(packageName: String): String {
        return packageName.replace('.', '/')
    }

    /**
     * Add the class designated by the fully qualified class name provided to the set of
     * resolved classes if and only if it is approved by the Test supplied.
     *
     * @param test the test used to determine if the class matches
     * @param fqn  the fully qualified name of a class
     */
    protected fun addIfMatching(test: Test, fqn: String) {
        try {
            val externalName = fqn.substring(0, fqn.indexOf('.')).replace('/', '.')
            val loader = classLoader
            if (loader == null) {
                return
            }

            if (log.isDebugEnabled) {
                log.debug("Checking to see if class $externalName matches criteria [$test]")
            }

            val type = loader.loadClass(externalName)
            if (test.matches(type)) {
                matches.add(type as Class<T>)
            }
        } catch (t: Throwable) {
            log.warn("Could not examine class '" + fqn + "'" + " due to a " +
                    t.javaClass.name + " with message: " + t.message)
        }

    }

    companion object {

        /*
     * An instance of Log to use for logging in this class.
     */
        private val log = LoggerFactory.getLogger(ClassResolver::class.java)
    }
}