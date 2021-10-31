package matthew.configuration.microprofile.config.converter;

import org.eclipse.microprofile.config.spi.Converter;

/**
 *  {@link Class} {@link Converter} implementation
 *
 * @author Matthew
 * @date 2021-10-31 13:31
 */
public class ClassConverter extends AbstractConverter<Class>
{
	private final ClassLoader classLoader;

	public ClassConverter()
	{
		this(Thread.currentThread().getContextClassLoader());
	}

	public ClassConverter(ClassLoader classLoader)
	{
		this.classLoader = classLoader;
	}

	@Override protected Class doConvert(String value) throws Throwable
	{
		return classLoader.loadClass(value);
	}
}
