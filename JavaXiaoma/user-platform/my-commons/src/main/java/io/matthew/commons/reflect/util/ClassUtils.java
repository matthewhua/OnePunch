package io.matthew.commons.reflect.util;

import java.lang.annotation.Annotation;

/**
 *
 *{@link Class} Utilities class
 *
 * @author Matthew
 * @date 2021-10-31 20:34
 */
public class ClassUtils
{
	public ClassUtils()
	{
	}

	public static <A extends Annotation> A findAnnotation(Class<?> type, Class<A> annotationType)
	{
		if (Object.class.equals(type) || type == null) {
			return null;
		}
		A annotation = type.getAnnotation(annotationType);
		if (annotation == null) {
			// find the annotation from the super interfaces
			for (Class<?> interfaceType : type.getInterfaces()) {
				annotation = interfaceType.getAnnotation(annotationType);
				if (annotation != null) {
					break;
				}
			}
		}

		if (annotation == null) {
			// find the annotation from the super class recursively
			annotation = findAnnotation(type.getSuperclass(), annotationType);
		}
		return annotation;
	}
}
