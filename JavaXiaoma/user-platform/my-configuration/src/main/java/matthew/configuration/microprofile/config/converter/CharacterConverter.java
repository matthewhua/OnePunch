package matthew.configuration.microprofile.config.converter;

/**
 * @author Matthew
 * @date 2021-10-31 13:29
 */
public class CharacterConverter extends AbstractConverter<Character>
{
	@Override protected Character doConvert(String value) throws Throwable
	{
		if (value == null || value.isEmpty())
			return null;
		return Character.valueOf(value.charAt(0));
	}
}
