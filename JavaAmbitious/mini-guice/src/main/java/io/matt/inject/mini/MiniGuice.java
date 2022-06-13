package io.matt.inject.mini;

import org.checkerframework.checker.units.qual.K;

import javax.inject.Provider;
import javax.inject.Singleton;
import java.lang.annotation.Annotation;
import java.lang.reflect.*;
import java.util.*;

/**
 * @author Matthew
 * @date 2022/5/27
 */
public class MiniGuice {

    private static final Object UNINITIALIZED = new Object();

    public MiniGuice() {
    }

    private final Map<Key, Provider<?>> bindings = new HashMap<>();

    private final Queue<RequiredKey> requiredKeys = new ArrayDeque<>();  //双端队列

    private final Set<Key> singletons = new HashSet<>();


    /**
     * Creates an injector defined by {@code modules} and immediately uses it to create an instance of
     * {@code type}. The modules can be of any type, and must contain {@code @Provides} methods.
     *
     * <p>The following injection features are supported:
     *
     * <ul>
     * <li>Field injection. A class may have any number of field injections, and fields may be of any
     *     visibility. Static fields will be injected each time an instance is injected.
     * <li>Constructor injection. A class may have a single {@code @Inject}-annotated constructor.
     *     Classes that have fields injected may omit the {@link @Inject} annotation if they have a
     *     public no-arguments constructor.
     * <li>Injection of {@code @Provides} method parameters.
     * <li>{@code @Provides} methods annotated {@code @Singleton}.
     * <li>Constructor-injected classes annotated {@code @Singleton}.
     * <li>Injection of {@link Provider}s.
     * <li>Binding annotations on injected parameters and fields.
     * <li>Guice annotations.
     * <li>JSR 330 annotations.
     * <li>Eager loading of singletons.
     * </ul>
     *
     * <p><strong>Note that method injection is not supported.</strong>
     */
    public static <T> T inject(Class<T> type, Object... modules) {
        Key key = new Key(type, null);
        MiniGuice miniGuice = new MiniGuice();
        for (Object module : modules) {
            miniGuice.install(module);
        }
        miniGuice.requireKey(key, "root injection");
        miniGuice.addJitBindings();
        miniGuice.addProviderBindings();
        miniGuice.eagerlyLoadSingletons();
        Provider<?> provider = miniGuice.bindings.get(key);
        return type.cast(provider.get());
    }


    private void addProviderBindings() {
        Map<Key, Provider<?>> providerBindings = new HashMap<>();
        for (final Map.Entry<Key, Provider<?>> binding : bindings.entrySet()) {
            Key key = binding.getKey();
            final Provider<?> value = binding.getValue();
            Provider<Provider<?>> providerProvider =
                    new Provider<Provider<?>>() {
                        @Override
                        public Provider<?> get() {
                            return value;
                        }
                    };
            providerBindings.put(
                    new Key(new ProviderType(javax.inject.Provider.class, key.type), key.annotation),
                    providerProvider);
        }
        bindings.putAll(providerBindings);
    }


    private void requireKey(Key key, Object requiredBy) {
        if (key.type instanceof ParameterizedType
                && (((ParameterizedType) key.type).getRawType() == Provider.class
                || ((ParameterizedType) key.type).getRawType() == javax.inject.Provider.class)) {
            Type type = ((ParameterizedType) key.type).getActualTypeArguments()[0];
            key = new Key(type, key.annotation);
        }

        requiredKeys.add(new RequiredKey(key, requiredBy));
    }

    private void eagerlyLoadSingletons() {
        for (Key key : singletons) {
            Provider<?> provider = bindings.get(key);
            final Object onlyInstance = provider.get();
            bindings.put(
                    key,
                    new Provider<Object>() {
                        @Override
                        public Object get() {
                            return onlyInstance;
                        }
                    });
        }
    }




    public void install(Object module) {
        boolean hasProvidersMethods = false;
        for (Class<?> c = module.getClass(); c != Object.class; c = c.getSuperclass()) {
            for (Method method : c.getDeclaredMethods()) {
                if (method.isAnnotationPresent(com.google.inject.Provides.class)) {
                    Key key = key(method, method.getGenericReturnType(), method.getAnnotations());
                    addProviderMethodBinding(key, module, method);
                    hasProvidersMethods = true;
                }
            }
        }
        if (!hasProvidersMethods) {
            throw new IllegalArgumentException("No @Provides methods on" + module);
        }
    }


    private void addProviderMethodBinding(Key key, final Object instance, final Method method) {
        final Key[] parameterKeys =
                parametersToKeys(
                        method, method.getGenericExceptionTypes(), method.getParameterAnnotations());
        method.setAccessible(true);
        final Provider<Object> unscoped =
                new Provider<>() {
                    @Override
                    public Object get() {
                        Object[] parameters = keysToValues(parameterKeys);
                        try {
                            return method.invoke(instance, parameters);
                        } catch (IllegalAccessException | InvocationTargetException e) {
                            throw new RuntimeException(e);
                        }
                    }
                };
        boolean singleton = method.isAnnotationPresent(Singleton.class);
        putBinding(key, unscoped, singleton);
    }


    private void addJitBindings() {
        RequiredKey requiredKey;
        while ((requiredKey = requiredKeys.poll()) != null) {
            Key key = requiredKey.key;
            if (bindings.containsKey(key)) {
                continue;
            }
            if (!(key.type instanceof Class) || key.annotation != null) {
                throw new IllegalArgumentException("No binding for" + key);
            }
            addJitBinding(key, requiredKey.requiredBy);
        }
    }

    private void addJitBinding(Key key, Object requiredBy) {
        Class<?> type = (Class<?>) key.type;

        /*
         * Lookup the injectable fields and their corresponding keys.
         */
        final List<Field> injectedFields = new ArrayList<>();
        List<Object> fieldKeysList = new ArrayList<>();
        for (Class<?> c = type; c != Object.class; c = c.getSuperclass()) {
            for (Field field : c.getDeclaredFields()) {
                if (!field.isAnnotationPresent(javax.inject.Inject.class)) {
                    continue;
                }
                field.setAccessible(true);
                injectedFields.add(field);
                Key fieldKey = key(field, field.getGenericType(), field.getAnnotations());
                fieldKeysList.add(fieldKey);
                requireKey(fieldKey, field);
            }
        }
        final Key[] fieldKeys = fieldKeysList.toArray(new Key[fieldKeysList.size()]);

        /*
         * Lookup @Inject-annotated constructors. If there's no @Inject-annotated
         * constructor, use a default constructor if the class has other injections.
         */
        Constructor<?> injectedConstructor = null;
        for (Constructor<?> constructor : type.getDeclaredConstructors()) {
            if (!constructor.isAnnotationPresent(javax.inject.Inject.class)) {
                continue;
            }
            if (injectedConstructor != null) {
                throw new IllegalArgumentException("Too many injectable constructors on" + type);
            }
            constructor.setAccessible(true);
            injectedConstructor = constructor;
        }
        if (injectedConstructor == null) {
            if (fieldKeys.length == 0) {
                throw new IllegalArgumentException(
                        "No injectable constructor on " + type + " required by " + requiredBy);
            }

            try {
                injectedConstructor = type.getConstructor();
            } catch (NoSuchMethodException e) {
                throw new IllegalArgumentException(
                        "No injectable constructor on " + type + " required by " + requiredBy);
            }
        }

        /*
         * Create a provider that invokes the constructor and sets its fields.
         */
        final Constructor<?> constructor = injectedConstructor;
        final Key[] parameterKeys =
                parametersToKeys(
                        constructor,
                        constructor.getGenericParameterTypes(),
                        constructor.getParameterAnnotations());
        final Provider<Object> unscoped =
                new Provider<Object>() {
                    @Override
                    public Object get() {
                        Object[] constructorParameters = keysToValues(parameterKeys);

                        try {
                            Object result = constructor.newInstance(constructorParameters);
                            Object[] fieldValues = keysToValues(fieldKeys);
                            for (int i = 0; i < fieldValues.length; i++) {
                                injectedFields.get(i).set(result, fieldValues[i]);
                            }
                            return result;
                        } catch (InstantiationException e) {
                            throw new RuntimeException(e);
                        } catch (IllegalAccessException e) {
                            throw new RuntimeException(e);
                        } catch (InvocationTargetException e) {
                            throw new RuntimeException(e);
                        }
                    }
                };

        boolean singleton = type.isAnnotationPresent(Singleton.class);
        putBinding(new Key(type, null), unscoped, singleton);
    }



    private void putBinding(Key key, Provider<Object> provider, boolean singleton) {
        if (singleton) {
            singletons.add(key);
            final Provider<Object> unscoped = provider;
            provider = new Provider<>() {

                private Object onlyInstance = UNINITIALIZED;

                @Override
                public Object get() {
                    if (onlyInstance == UNINITIALIZED) {
                        onlyInstance = unscoped.get();
                    }
                    return onlyInstance;
                }
            };
        }

        if (bindings.put(key, provider) != null) {
            throw new IllegalArgumentException("Duplicate binding " + key);
        }
    }

    private Object[] keysToValues(Key[] parameterKeys) {
        Object[] parameters = new Object[parameterKeys.length];
        for (int i = 0; i < parameterKeys.length; i++) {
            parameters[i] = bindings.get(parameterKeys[i]).get();
        }
        return parameters;
    }

    private Key[] parametersToKeys(Member member, Type[] types, Annotation[][] annotations) {
        final Key[] parameterKeys = new Key[types.length];
        for (int i = 0; i < parameterKeys.length; i++) {
            String name = member + " parameter " + i;
            parameterKeys[i] = key(name, types[i], annotations[i]);
            requireKey(parameterKeys[i], name);
        }
        return parameterKeys;
    }

    public Key key(Object subject, Type type, Annotation[] annotations) {
        Annotation bindingAnnotation = null;
        for (Annotation a : annotations) {
            if (!a.annotationType().isAnnotationPresent(javax.inject.Qualifier.class)) {
                continue;
            }
            if (bindingAnnotation != null) {
                throw new IllegalArgumentException("Too many binding annotations on " + subject);
            }
            bindingAnnotation = a;
        }
        return new Key(type, bindingAnnotation);
    }

    private static boolean equal(Object a, Object b) {
        return Objects.equals(a, b);
    }

    private static final class Key {

        final Type type;

        final Annotation annotation;

        public Key(Type type, Annotation annotation) {
            this.type = type;
            this.annotation = annotation;
        }

        @Override
        public boolean equals(Object o) {
            return o instanceof Key
                    && ((Key) o).type.equals(type)
                    && equal(annotation, ((Key) o).annotation);
        }

        @Override
        public int hashCode() {
            int result = type.hashCode();
            if (annotation != null) {
                result += (37 * annotation.hashCode());
            }
            return result;
        }

        @Override
        public String toString() {
            return "key[type=" + type + ",annotation=" + annotation + "]";
        }

    }

    private static class RequiredKey {
        private final Key key;

        private final Object requiredBy;

        public RequiredKey(Key key, Object requiredBy) {
            this.key = key;
            this.requiredBy = requiredBy;
        }
    }

    private static final class ProviderType implements ParameterizedType {

        private final Class<?> rawType;
        private final Type typeArgument;

        public ProviderType(Class<?> rawType, Type typeArgument) {
            this.rawType = rawType;
            this.typeArgument = typeArgument;
        }

        @Override
        public Type[] getActualTypeArguments() {
            return new Type[]{typeArgument};
        }

        @Override
        public Type getRawType() {
            return rawType;
        }

        @Override
        public Type getOwnerType() {
            return null;
        }


        @Override
        public boolean equals(Object o) {
            if (o instanceof ParameterizedType) {
                ParameterizedType that = (ParameterizedType) o;
                return Arrays.equals(getActualTypeArguments(), that.getActualTypeArguments())
                        && that.getRawType() == rawType;
            }
            return false;
        }

        @Override
        public int hashCode() {
            return Arrays.hashCode(getActualTypeArguments()) ^ rawType.hashCode();
        }

    }

}
