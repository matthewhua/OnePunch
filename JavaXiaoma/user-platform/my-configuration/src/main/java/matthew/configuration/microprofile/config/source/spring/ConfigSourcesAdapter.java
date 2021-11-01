package matthew.configuration.microprofile.config.source.spring;

import org.eclipse.microprofile.config.spi.ConfigSource;
import org.springframework.core.env.ConfigurableEnvironment;
import org.springframework.core.env.EnumerablePropertySource;
import org.springframework.core.env.Environment;
import org.springframework.core.env.MutablePropertySources;

import java.util.Collections;
import java.util.LinkedList;
import java.util.List;

/**
 *  * {@link ConfigSource} Adapter
 *
 * @author Matthew
 * @time 2021/11/1 16:32
 */
public class ConfigSourcesAdapter {

    public List<ConfigSource> getConfigSources(Environment environment) {
        List<ConfigSource> configSourcesList = new LinkedList<>();
        if (environment instanceof ConfigurableEnvironment){
            final ConfigurableEnvironment configurableEnvironment = (ConfigurableEnvironment) environment;
            final MutablePropertySources propertySources = configurableEnvironment.getPropertySources();
            propertySources.stream()
                    .filter(propertySource -> propertySource instanceof EnumerablePropertySource)
                    .map(EnumerablePropertySource.class::cast)
                    .map(PropertySourceConfigSource::new)
                    .forEach(configSourcesList::add);
        }
        return Collections.unmodifiableList(configSourcesList);
    }
}
