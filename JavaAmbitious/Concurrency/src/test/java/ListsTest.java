import io.matt.util.Lists;
import org.junit.Test;

import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.Arrays;
import java.util.List;

/**
 * @author Matthew
 * @date 2022/5/6
 */
public class ListsTest {

    @Test
    public void test(){
        List<String> s = Lists.newArray();
        s.add("matt");
        ParameterizedType  genericSuperclass = (ParameterizedType)s.getClass().getGenericSuperclass(); //通过父类去找
        Type[] actualTypeArguments = genericSuperclass.getActualTypeArguments(); // E
        System.out.println(genericSuperclass);
        System.out.println(actualTypeArguments[0]);
        System.out.println(Arrays.toString(s.getClass().getDeclaredFields()));
    }
}
