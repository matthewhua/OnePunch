package matthew.configuration.microprofile.config;

import matthew.configuration.microprofile.config.source.ConfigSources;
import org.eclipse.microprofile.config.Config;
import org.eclipse.microprofile.config.ConfigValue;
import org.eclipse.microprofile.config.spi.ConfigSource;
import org.eclipse.microprofile.config.spi.Converter;

import java.util.Optional;

/**
 * @author Matthew
 * @time 2021/10/25 13:43
 */
public class DefaultConfig implements Config {

    private final ConfigSources configSources;

    private final Converter converters;

	public DefaultConfig(ConfigSources configSources, Converter converters)
	{
		this.configSources = configSources;
		this.converters = converters;
	}


	@Override
    public <T> T getValue(String propertyName, Class<T> propertyType) {
		ConfigValue configValue = getConfigValue(propertyName);
		if (configValue == null)
			return null;
		String propertyValue = configValue.getValue();
		// String 转换成目标类型
		doGetConverter(propertyType)

	}



	@Override
    public ConfigValue getConfigValue(String s) {
        return null;
    }

    @Override
    public <T> Optional<T> getOptionalValue(String s, Class<T> aClass) {
        return Optional.empty();
    }

    @Override
    public Iterable<String> getPropertyNames() {
        return null;
    }

    @Override
    public Iterable<ConfigSource> getConfigSources() {
        return null;
    }

    @Override
    public <T> Optional<Converter<T>> getConverter(Class<T> aClass) {
        return Optional.empty();
    }

	protected  <T> void doGetConverter(Class<T> propertyType)
	{
		this.converters.get
	}

    @Override
    public <T> T unwrap(Class<T> aClass) {
        return null;
    }
}
