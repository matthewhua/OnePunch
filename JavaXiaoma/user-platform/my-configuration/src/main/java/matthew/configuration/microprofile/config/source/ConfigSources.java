package matthew.configuration.microprofile.config.source;

import org.eclipse.microprofile.config.spi.ConfigSource;

import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;

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
        if(a)
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


}
