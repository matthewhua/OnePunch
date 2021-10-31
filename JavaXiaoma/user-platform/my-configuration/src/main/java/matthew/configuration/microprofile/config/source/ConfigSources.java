package matthew.configuration.microprofile.config.source;

import org.eclipse.microprofile.config.spi.ConfigSource;

import java.util.Arrays;
import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;

import static java.util.Collections.sort;
import static jdk.nashorn.internal.objects.Global.load;

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


    public void addDeFaultSources() {
        if(addedDefaultConfigSources)
        	return;

        addC

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
    	addC
	}

	public void  addConfigSources(ConfigSource... configSources)
	{
		addConfigSources(Arrays.asList(configSources));
	}

	private  void addConfigSources(Iterable<ConfigSource>  configSourceClass)
	{
		configSources.forEach(this.configSources::add);
		sort(this.configSources, ConfigSource);
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
