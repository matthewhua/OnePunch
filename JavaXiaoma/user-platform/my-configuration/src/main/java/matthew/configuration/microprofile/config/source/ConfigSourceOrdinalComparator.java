package matthew.configuration.microprofile.config.source;

import org.eclipse.microprofile.config.spi.ConfigSource;

import java.util.Comparator;

/**
 * @author Matthew
 * @date 2021-10-31 17:20
 * {@link ConfigSource} 优先级比较器
 */
public class ConfigSourceOrdinalComparator implements Comparator<ConfigSource>
{
	/**
	 * Singleton instance {@link ConfigSourceOrdinalComparator}
	 */
	public static final Comparator<ConfigSource> INSTANCE = new ConfigSourceOrdinalComparator();

	private ConfigSourceOrdinalComparator() {
	}


	@Override public int compare(ConfigSource o1, ConfigSource o2)
	{
		return Integer.compare(o2.getOrdinal(), o1.getOrdinal());
	}
}
