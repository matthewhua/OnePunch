package xyz.ariane.util.xml

import org.w3c.dom.Document
import org.xml.sax.InputSource
import java.io.StringReader
import javax.xml.parsers.DocumentBuilderFactory
import javax.xml.parsers.ParserConfigurationException

fun buildXmlDocumentFromText(xmlText: String): Document {
    val docBuilder = XmlUtils.newDocFactoryNoValidation().newDocumentBuilder()
    return StringReader(xmlText).use { docBuilder.parse(InputSource(it)) }
}

/**
 * Internal API
 */
internal object XmlUtils {

    /**
     * Xerces features prefix ("http://apache.org/xml/features/").
     */
    private val XERCES_FEATURE_PREFIX = "http://apache.org/xml/features/"

    /**
     * Load external dtd when nonvalidating feature ("nonvalidating/load-external-dtd").
     */
    private val LOAD_EXTERNAL_DTD_FEATURE = "nonvalidating/load-external-dtd"

    private val LOAD_EXT_DTD_FEATURE = XERCES_FEATURE_PREFIX + LOAD_EXTERNAL_DTD_FEATURE

    @Throws(ParserConfigurationException::class)
    fun newDocFactoryNoValidation(): DocumentBuilderFactory {
        val factory = DocumentBuilderFactory.newInstance()
        factory.isValidating = false
        factory.setFeature(LOAD_EXT_DTD_FEATURE, false)
        return factory
    }

}
