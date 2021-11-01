package matthew.configuration.microprofile.config.source;

import java.io.InputStream;
import java.net.URL;
import java.util.Map;
import java.util.Properties;
import java.util.logging.Logger;

/**
 * @author Matthew
 * @time 2021/11/1 11:45
 */
public class DefaultResourceConfigSource extends MapBasedConfigSource{

    private static final String configFileLocation = "META-INF/microprofile-config.properties";

    private final Logger logger = Logger.getLogger(this.getClass().getName());

    public DefaultResourceConfigSource() {
        super("Default Config File", 100);
    }

    @Override
    protected void prepareConfigData(Map configData) throws Throwable {
        final ClassLoader classLoader = getClass().getClassLoader();
        final URL resource = classLoader.getResource(configFileLocation);
        if (resource == null){
            logger.info("the default config file can't be fount in the classpath : " + configFileLocation);
            return;
        }
        try (final InputStream inputStream = resource.openStream()){
            final Properties properties = new Properties();
            properties.load(inputStream);
            configData.putAll(properties);
        }
    }

}
