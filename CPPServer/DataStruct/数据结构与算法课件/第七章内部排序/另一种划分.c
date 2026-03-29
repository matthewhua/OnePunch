#include <stdio.h>
#include <stdlib.h>

typedef int ELEMType;

// 记录结构
typedef struct {
    ELEMType *data;
    int currentNum;
} myRcd;

int partition1(myRcd *myarr, int mostleft, int mostright)  // 转移划分
{
    int left = mostleft, right = mostright, flag = 0;
    ELEMType pivot;
    
    if (mostleft < mostright) {
        pivot = myarr->data[mostleft];  // 第一个元素选做枢轴
        
        for (;;) {  // 循环查找左边大于枢轴、右边小于枢轴的元素
            while (myarr->data[right] >= pivot && right > (mostleft)) {
                right--;
                flag = 0;
            }
            // 找到右边第一个小于枢轴的记录
            if (left < right) 
                myarr->data[left++] = myarr->data[right];
            
            while (myarr->data[left] <= pivot && left < mostright) {
                left++;
                flag = 1;
            }
            // 找到左边第一个大于枢轴的记录
            if (left < right) 
                myarr->data[right--] = myarr->data[left];
            
            if (left >= right)
                break;
        }
        
        if (flag == 1) 
            myarr->data[right] = pivot;
        else 
            myarr->data[left] = pivot;
    }
    
    return right;
}
