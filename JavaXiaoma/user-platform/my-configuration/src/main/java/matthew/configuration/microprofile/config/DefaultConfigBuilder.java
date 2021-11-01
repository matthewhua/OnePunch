package matthew.configuration.microprofile.config;

import matthew.configuration.microprofile.config.converter.Converters;
import matthew.configuration.microprofile.config.source.ConfigSources;
import org.eclipse.microprofile.config.Config;
import org.eclipse.microprofile.config.spi.ConfigBuilder;
import org.eclipse.microprofile.config.spi.ConfigSource;
import org.eclipse.microprofile.config.spi.Converter;

/**
 * @author Matthew
 * @time 2021/11/1 10:27
 * @see {@link ConfigSources}
 */
public class DefaultConfigBuilder implements ConfigBuilder {

    private final ConfigSources configSources;

    private final Converters converters;

    public DefaultConfigBuilder(ConfigSources configSources, Converters converters) {
        this.configSources = configSources;
        this.converters = converters;
    }

    @Override
    public ConfigBuilder addDefaultSources() {
        configSources.addConfigSources();
        return this;
    }

    @Override
    public ConfigBuilder addDiscoveredSources() {
        return null;
    }

    @Override
    public ConfigBuilder addDiscoveredConverters() {
        return null;
    }

    @Override
    public ConfigBuilder forClassLoader(ClassLoader classLoader) {
        return null;
    }

    @Override
    public ConfigBuilder withSources(ConfigSource... configSources) {
        return null;
    }

    @Override
    public ConfigBuilder withConverters(Converter<?>... converters) {
        return null;
    }

    @Override
    public <T> ConfigBuilder withConverter(Class<T> aClass, int i, Converter<T> converter) {
        return null;
    }

    @Override
    public Config build() {
        return null;
    }
}
