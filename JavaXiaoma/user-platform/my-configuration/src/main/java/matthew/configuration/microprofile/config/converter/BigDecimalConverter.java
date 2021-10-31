package matthew.configuration.microprofile.config.converter;

import java.math.BigDecimal;

/**
 * @author Matthew
 * @date 2021-10-31 13:28
 */
public class BigDecimalConverter extends AbstractConverter<BigDecimal>
{
	@Override protected BigDecimal doConvert(String value) throws Throwable
	{
		return new BigDecimal(value);
	}
}
