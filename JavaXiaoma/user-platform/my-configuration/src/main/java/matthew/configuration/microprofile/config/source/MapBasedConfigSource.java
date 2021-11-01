package matthew.configuration.microprofile.config.source;

import org.eclipse.microprofile.config.spi.ConfigSource;

import java.util.HashMap;
import java.util.Map;
import java.util.Set;

/**
 * @author Matthew
 * @time 2021/11/1 11:27
 * 基于 Map 数据结构 {@link ConfigSource} 实现
 */
public abstract class MapBasedConfigSource implements ConfigSource {

    private final String name;

    private final int ordinal;

    private final Map<String, String> configData;

    protected MapBasedConfigSource(String name, int ordinal) {
        this.name = name;
        this.ordinal = ordinal;
        this.configData = new HashMap<>();
    }


    protected Map<String, String> getConfigData(){

        try {
            if (configData.isEmpty()){
                prepareConfigData(configData);
            }
        } catch (Throwable throwable) {
            throw new IllegalStateException("准备配置数据发生错误", throwable);
        }
        return configData;
    }


    /**
     * 准备配置数据
     *
     * @param configData
     * @throws Throwable
     */
    protected abstract void prepareConfigData(Map configData) throws Throwable;


    @Override
    public Set<String> getPropertyNames() {
        return configData.keySet();
    }

    @Override
    public String getValue(String propertyName) {
        return getConfigData().get(propertyName);
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public final int getOrdinal() {
        return ordinal;
    }
}
