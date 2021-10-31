package io.matthew.commons.io;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.ObjectInputStream;

/**
 *
 * Default {@link Deserializer} based on Java Standard Serialization.
 *
 * @author Matthew
 * @date 2021-10-31 20:27
 */
public class DefaultDeserializer implements Deserializer<Object>
{

	@Override public Object deserialize(byte[] bytes) throws IOException
	{
		if (bytes == null)
			return null;
		Object value = null;
		try(ByteArrayInputStream inputStream = new ByteArrayInputStream(bytes);
				ObjectInputStream objectInputStream = new ObjectInputStream(inputStream)
		){
			// byte[] -> Value
			value = objectInputStream.readObject();
		}catch (Exception e){
			throw new IOException(e);
		}
			return value;
	}
}
