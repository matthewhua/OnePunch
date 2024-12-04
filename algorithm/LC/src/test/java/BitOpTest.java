import org.junit.Test;


public class BitOpTest {

    @Test
    public void BitXor() {
        int a = 5, b = 3;
        a = a ^ b;
        b = a ^ b;
        a = a ^ b;


        System.out.println(a);
        System.out.println(b);
    }
}
