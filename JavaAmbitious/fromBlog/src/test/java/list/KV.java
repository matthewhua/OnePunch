package list;

/**
 * @author Matthew
 * @date 2022/5/10
 */
public class KV <K, V>{
    K k;
    V v;

    public KV(K k, V v) {
        this.k = k;
        this.v = v;
    }

    public KV() {

    }

    public K getK() {
        return k;
    }

    public void setK(K k) {
        this.k = k;
    }

    public V getV() {
        return v;
    }

    public void setV(V v) {
        this.v = v;
    }

    @Override
    public String toString() {
        return "KV{" +
                "k=" + k +
                ", v=" + v +
                '}';
    }
}
