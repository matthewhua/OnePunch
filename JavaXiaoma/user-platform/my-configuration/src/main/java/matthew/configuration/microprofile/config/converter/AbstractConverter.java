package matthew.configuration.microprofile.config.converter;

import org.eclipse.microprofile.config.spi.Converter;

/**
 * @author Matthew
 * @date 2021-10-31 13:23
 */
public abstract class AbstractConverter<T> implements Converter<T>
{
	@Override public T convert(String value) throws IllegalArgumentException, NullPointerException
	{
		if (value == null)
			throw new NullPointerException("the value must not be null");
		T convertedValue = null;
		try
		{
			convertedValue = doConvert(value);
		}
		catch (Throwable e)
		{
			throw new IllegalArgumentException("The value can't be converted.", e);
		}
		return convertedValue;
	}

	protected abstract T doConvert(String value) throws Throwable;
}
