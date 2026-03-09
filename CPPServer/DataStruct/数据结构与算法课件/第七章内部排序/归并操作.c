#include <stdio.h>
#include <stdlib.h>

typedef int ELEMType;

// 记录结构
typedef struct {
    ELEMType *data;
    int currentNum;
} myRcd;

void merge(myRcd *myarr, myRcd *tmplist, int l, int m, int n)
{
    int i = l, j = m + 1, k = l - 1, t;
    
    while (i <= m && j <= n) {
        if (myarr->data[i] <= myarr->data[j])  // 将两个子段中较小记录移到临时空间中
            tmplist->data[++k] = myarr->data[i++];
        else
            tmplist->data[++k] = myarr->data[j++];
    }
    
    if (i <= m)
        for (t = i; t <= m; t++)  // 将第一个子段中的剩余元素移到临时空间中
            tmplist->data[++k] = myarr->data[t];
    
    if (j <= n)
        for (t = j; t <= n; t++)  // 将第二个子段中的剩余元素移到临时空间中
            tmplist->data[++k] = myarr->data[t];
    
    for (i = l; i <= n; i++)  // 将临时空间中的记录移回到数组list中
        myarr->data[i] = tmplist->data[i];
}
