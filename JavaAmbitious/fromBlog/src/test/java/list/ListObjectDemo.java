package list;

import org.junit.Test;

import java.util.ArrayList;
import java.util.List;

/**
 * @author Matthew
 * @date 2022/5/10
 */
public class ListObjectDemo {


    @Test
    public void test01(){
        KV<Integer, String> kv = new KV<Integer, String>(); //每次得new 一个
        List<KV> as = new ArrayList<>();
        for (int i = 0; i < 3; i++) {
            kv.setK(i);
            kv.setV("matthew" + i);
            as.add(kv);
        }
        System.out.println(as);
    }
}
