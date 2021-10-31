package io.matthew.commons.io;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.ObjectOutputStream;
import java.io.Serializable;

/**
 * Default Serializer implementation based on Java Standard Serialization.
 *  @see ObjectOutputStream
 *  @see Serializable
 *
 * @author Matthew
 * @date 2021-10-31 20:26
 */
public class DefaultSerializer implements Serializer<Object>
{
	@Override
	public byte[] serialize(Object source) throws IOException
	{
		byte[] bytes = null;
		try (ByteArrayOutputStream outputStream = new ByteArrayOutputStream();
				ObjectOutputStream objectOutputStream = new ObjectOutputStream(outputStream)
		) {
			// Key -> byte[]
			objectOutputStream.writeObject(source);
			bytes = outputStream.toByteArray();
		}
		return bytes;
	}
}
