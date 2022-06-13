package com.matt.common.eventbus;

import com.google.inject.Provides;
import io.matt.inject.mini.MiniGuice;
import junit.framework.TestCase;

import javax.inject.Inject;
import javax.inject.Provider;
import javax.inject.Singleton;

/**
 * @author Matthew
 * @date 2022/6/13
 */
@SuppressWarnings("ProvidesMethodOutsideOfModule")
public class MiniGuiceTest extends TestCase {

    static class A{

        @Inject
        A() {
        }
    }


    static class B {
        @Inject
        B() {}
    }

    @Singleton
    static class C {
        @Inject
        C() {}
    }

    @Singleton
    static class D {
        @Inject
        D() {}
    }

    static class E {
        F f;

        E(F f) {
            this.f = f;
        }
    }

    static class F {}

    static class G{
        @Inject A a;

        @Inject B b;

        C c;

        D d;

        @Inject E e;

        @Inject
        G(C c, D d){
            this.c = c;
            this.d = d;
        }
    }

    public void testProviderInjection(){
        H h = MiniGuice.inject(H.class);
        assertNotNull(h.aProvider.get());
        assertNotNull(h.aProvider.get());
        assertNotSame(h.aProvider.get(), h.aProvider.get());
    }

    static class H {

        @Inject
        Provider<A> aProvider;

        @Inject
        H() {}
    }

    public void testSingletons(){
        J j =
                MiniGuice.inject(
                        J.class,
                        new Object(){
                            @Provides
                            @Singleton
                            F provideK(){
                                return new F();
                            }
                        });
        assertSame(j.fProvider.get(), j.fProvider.get());
        assertSame(j.iProvider.get(), j.iProvider.get());
    }

    @Singleton
    static class I {
        @Inject
        I() {}
    }


    static class J {
        @Inject Provider<F> fProvider;
        @Inject Provider<I> iProvider;

        @Inject
        J() {}
    }


}
