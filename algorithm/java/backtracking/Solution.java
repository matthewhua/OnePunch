import java.util.ArrayList;

public class Solution {
    public boolean validateIp(String ipStr) {
        String[] ss = ipStr.split("\\.", -1);
        if (ss.length != 4) {
            return false;
        }
        for (String s : ss) {
            String trim = s.trim();
            if (trim.length() == 0) {
                return false;
            }
            if (trim.startsWith("0") && !trim.equals("0")) {
                return false;
            }
            if (trim.contains(" ")) {
                return false;
            }
            int i = Integer.parseInt(trim);
            if (i < 0 || i > 255) {
                return false;
            }
        }
        return true;
    }

    static class ListNode {
        public int val;

        public ListNode next;

        public ListNode() {
        }

        public ListNode(int val) {
            this.val = val;
        }

        public ListNode(int val, ListNode next) {
            this.val = val;
            this.next = next;
        }
    }

    public ListNode reorder(ListNode head) {
        if (head == null || head.next == null) {
            return null;
        }

        ListNode slow = head, fast = head;
        while (fast.next != null && fast.next.next != null) {
            slow = slow.next;
            fast = fast.next.next;
        }

        // 步骤2
        ListNode secondHalf = reverseList(slow.next);
        slow.next = null;

        mergeList(head, secondHalf);
        return head;
    }

    private ListNode reverseList(ListNode head) {
        ListNode prev = null;
        ListNode current = head;

        whi
            ListNode next = current.next;
            current.next = prev;
            prev = current;
            current = next;
        }
        return prev;
    }

    private void mergeList(ListNode l1, ListNode l2) {
        while (l1 != null && l2 != null) {
            ListNode l1Next = l1.next;
            ListNode l2Next = l2.next;

            l1.next = l2;
            l2.next = l1Next;

            l1 = l1Next;
            l2 = l2Next;
        }
    }


    public static void main(String[] args) {
        Solution solution = new Solution();
        ListNode listNode = new ListNode(1);
        ListNode listNode1 = new ListNode(3);
        ListNode listNode2 = new ListNode(2);
        ListNode listNode3 = new ListNode(5);
        ListNode listNode4 = new ListNode(7);
        listNode.next = listNode1;
        listNode1.next = listNode2;
        listNode2.next = listNode3;
        listNode3.next = listNode4;
        solution.reorder(listNode);
    }
}