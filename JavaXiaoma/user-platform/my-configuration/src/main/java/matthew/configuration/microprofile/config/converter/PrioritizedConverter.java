package matthew.configuration.microprofile.config.converter;

import org.eclipse.microprofile.config.spi.Converter;

/**
 * @author Matthew
 * @date 2021-10-31 13:33
 */
public class PrioritizedConverter<T> implements Converter<T>, Comparable<PrioritizedConverter<T>>
{
	private final Converter<T> converter;

	private final int priority;

	public PrioritizedConverter(Converter<T> converter, int priority)
	{
		this.converter = converter;
		this.priority = priority;
	}

	public int getPriority() {
		return priority;
	}

	public Converter<T> getConverter() {
		return converter;
	}


	@Override public int compareTo(PrioritizedConverter<T> other)
	{
		return Integer.compare(other.getPriority(), this.getPriority());
	}

	@Override public T convert(String s) throws IllegalArgumentException, NullPointerException
	{
		return converter.convert(s);
	}
}
