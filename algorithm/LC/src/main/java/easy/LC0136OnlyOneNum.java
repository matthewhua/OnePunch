package easy;

import java.util.HashMap;
import java.util.Map;

public class LC0136OnlyOneNum {


    /**
     * 自己的解法
     *
     * @param nums
     * @return 0
     */
    public int singleNumber(int[] nums) {
        HashMap<Integer, Integer> set = new HashMap<>(4);
        for (int num : nums) {
            set.put(num, set.getOrDefault(num, 0) + 1);
        }
        for (Map.Entry<Integer, Integer> entry : set.entrySet()) {
            if (entry.getValue() == 1) {
                return entry.getKey();
            }
        }
        return -1;
    }


    public int singleNumber2(int[] nums) {
        int res = 0;
        for (int num : nums) {
            res ^= num;
        }
        return res;
    }
}
