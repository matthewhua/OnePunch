package io.matthew.commons.reflect.util;

import java.lang.reflect.Type;
import java.lang.reflect.TypeVariable;

/**
 * @author Matthew
 * @date 2021-10-31 22:35
 */
public class TypeUtils
{
	public TypeUtils()
	{
	}



	private static Class<?> asClass(Type typeArgument)
	{
		if (typeArgument instanceof Class){
			return (Class<?>) typeArgument;
		}else if (typeArgument instanceof TypeVariable){
			TypeVariable typeVariable = (TypeVariable) typeArgument;
			return asClass(typeVariable.getBounds()[0]);
		}
		return null;
	}
}
