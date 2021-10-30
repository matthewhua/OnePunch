package matthew.configuration.microprofile.config;


import org.eclipse.microprofile.config.ConfigValue;

/**
 * @author Matthew
 * @time 2021/10/25 9:54
 * 默认实现 {@link ConfigValue}
 */
class DefaultConfigValue implements ConfigValue{

    private final String name;

    private final String value;

    private final String rawValue;

    private final String sourceName;

    private final int sourceOrdinal;


    public DefaultConfigValue(String name, String value, String rawValue, String sourceName, int sourceOrdinal) {
        this.name = name;
        this.value = value;
        this.rawValue = rawValue;
        this.sourceName = sourceName;
        this.sourceOrdinal = sourceOrdinal;
    }

    @Override
    public String getName() {
        return null;
    }

    @Override
    public String getValue() {
        return null;
    }

    @Override
    public String getRawValue() {
        return null;
    }

    @Override
    public String getSourceName() {
        return null;
    }

    @Override
    public int getSourceOrdinal() {
        return 0;
    }
}
