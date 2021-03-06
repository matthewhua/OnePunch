package matthew.configuration.microprofile.config.source;

import org.eclipse.microprofile.config.spi.ConfigSource;

import java.util.Arrays;
import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;

import static java.util.Collections.sort;
import static java.util.ServiceLoader.load;
import static java.util.stream.Stream.of;
/**
 * @author Matthew
 * @time 2021/10/25 13:44
 */
public class ConfigSources implements Iterable<ConfigSource>{

    private boolean addedDefaultConfigSources;

    private boolean addedDiscoveredConfigSources;

    private List<ConfigSource> configSources = new LinkedList<>();

    private ClassLoader classLoader;

    public ConfigSources(ClassLoader classLoader) {
        this.classLoader = classLoader;
    }

    public void  setConfigSources(ClassLoader classLoader) {
        this.classLoader = classLoader;
    }


	public void setClassLoader(ClassLoader classLoader) {
		this.classLoader = classLoader;
	}


    public void addDeFaultSources() {
        if(addedDefaultConfigSources)
        	return;
		addConfigSources(JavaSystemPropertiesConfigSource.class,
				OperationSystemEnvironmentVariablesConfigSource.class,
				DefaultResourceConfigSource.class
		);

    }


	public void addDiscoveredSources()
	{
		if (addedDiscoveredConfigSources)
		{
			return;
		}

		addConfigSources(load(ConfigSource.class, classLoader));

	}


	public void addConfigSources(Class<? extends ConfigSource>... configSourceClasses) {
		addConfigSources(
				of(configSourceClasses)
						.map(this::newInstance)
						.toArray(ConfigSource[]::new)
		);
	}

	public void  addConfigSources(ConfigSource... configSources)
	{
		addConfigSources(Arrays.asList(configSources));
	}

	private  void addConfigSources(Iterable<ConfigSource>  configSourceClass)
	{
		configSources.forEach(this.configSources::add);
		sort(this.configSources,  ConfigSourceOrdinalComparator.INSTANCE);
	}

	private ConfigSource newInstance(Class<? extends ConfigSource> configSourceClass) {
		ConfigSource instance = null;
		try {
			instance = configSourceClass.newInstance();
		} catch (InstantiationException | IllegalAccessException e) {
			throw new IllegalStateException(e);
		}
		return instance;
	}

	@Override
    public Iterator<ConfigSource> iterator() {
        return configSources.iterator();
    }


    public boolean isAddedDefaultConfigSources() {
        return addedDefaultConfigSources;
    }

    public boolean isAddedDiscoveredConfigSources() {
        return addedDiscoveredConfigSources;
    }

	public ClassLoader getClassLoader() {
		return classLoader;
	}
}
