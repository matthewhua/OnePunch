package io.matthew.commons.io;

import java.io.IOException;

/**
 * @author Matthew
 * @date 2021-10-31 20:25
 *
 * @param <T> the type to be deserialized
 */
public interface Deserializer<T>
{

	T deserialize(byte[] bytes) throws IOException;
}
