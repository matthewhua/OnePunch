package io.matt.behavior.strategy.sort;

public class Main {

    public static void main(String[] args) {
        SortContext sortContext = new SortContext();
        int arr[] = { 10, 23, -1, 0, 300, 87, 28, 77, -32, 2 };
        sortContext.setInput(arr, arr.length);
        System.out.print("input:");

        // BubbleSort
        sortContext.setStrategy(new BubbleSort());
        sortContext.sort();


        // SelectionSort
        sortContext.setStrategy(new SelectionSort());
        sortContext.sort();

        // InsertSort
        sortContext.setStrategy(new InsertSort());
        sortContext.sort();

    }
}
