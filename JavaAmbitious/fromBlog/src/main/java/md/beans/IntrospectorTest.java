package md.beans;

import java.beans.BeanInfo;
import java.beans.IntrospectionException;
import java.beans.Introspector;
import java.beans.PropertyDescriptor;

/**
 * @author Matthew
 * @date 2022/5/7
 */
public class IntrospectorTest {

    /**
     * Reflection：反射就是运行时获取一个类的所有信息，可以获取到类的所有定义的信息（包括成员变量，成员方法，构造器等）可以操纵类的字段、方法、构造器等部分。可以想象为镜面反射或者照镜子，这样的操作是带有客观色彩的，也就是反射获取到的类信息是必定正确的。
     * Introspector：内省基于反射实现，主要用于操作JavaBean，基于JavaBean的规范进行Bean信息描述符的解析，依据于类的Setter和Getter方法，可以获取到类的描述符。可以想象为“自我反省”，这样的操作带有主观的色彩，不一定是正确的(如果一个类中的属性没有Setter和Getter方法，无法使用内省)。
     *
     *
     * 作者: Throwable
     * 链接: https://throwx.cn/2019/12/25/java-introspector-usage/
     * @param args
     */
    public static void main(String[] args) throws IntrospectionException {
        BeanInfo beanInfo = Introspector.getBeanInfo(Person.class);
        PropertyDescriptor[] propertyDescriptors = beanInfo.getPropertyDescriptors();
        for (PropertyDescriptor propertyDescriptor : propertyDescriptors) {
            if (!"class".equals(propertyDescriptor.getName())){
                System.out.println(propertyDescriptor.getName());
                System.out.println(propertyDescriptor.getWriteMethod().getName());
                System.out.println(propertyDescriptor.getReadMethod().getName());
                System.out.println("========================================");
            }
        }
    }

    public static class Person{
        private Long id;
        private String name;
        private Integer age;

        public Long getId() {
            return id;
        }

        public void setId(Long id) {
            this.id = id;
        }

        public String getName() {
            return name;
        }

        public void setName(String name) {
            this.name = name;
        }

        public Integer getAge() {
            return age;
        }

        public void setAge(Integer age) {
            this.age = age;
        }
    }
}
