//
// Created by Admin on 2021/11/15.
//
#include<iostream>
using namespace std;
#include<list>
#include<vector>
#include<queue>
#include<string>
#include<ctime>

//������Ա
class Student
{
public:
    string name;
};

// ��ӡ��Ա
void printVector(vector<Student> &vec)
{
    for (auto & it : vec) {
        cout << it.name << endl;
    }
}

//������Ա
void CreateStudent(queue<Student> &que, int num)
{
    string setName = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    int sum = rand() % 10;
    for (int i = 0; i < sum; i++) {
        Student stu;
        char buf[64] = { 0 };
        sprintf(buf, "%d", num);
        string s(buf);

        stu.name = "��";
        stu.name += s;
        stu.name += "��";
        stu.name += setName[i];

        que.push(stu);  // ÿ�����Ա�����������
    }
}

//mylist�ǵ���,que��������,pushV�ǿ��������ݵ���Ա
int PushList(list<Student> &mylist, queue<Student> &que, vector<Student> &pushV)
{
    int tmpPush = 0; //��ʱ��������¼��������Ա��
    
    while (!que.empty())
    {
        if (mylist.size() >= 15) //��������
            break;

        Student s = que.front();

        //������vector
        pushV.push_back(s);

        //������
        mylist.push_back(s);

        //�����ж�ͷԪ�س�����
        que.pop();

        tmpPush++;
    }
    return tmpPush;
}

//mylist�ǵ���,popV��¼��������Ա��num����
int PopList(list<Student> &myList, vector<Student> &popV, int num)
{
    int tmppop = 0;
    if (num == 17)
    {
        while (!myList.empty())
        {
            Student s = myList.front();
            // �ѵ��ݵ��˿�����popV ��
            popV.push_back(s);

            myList.pop_front();
            tmppop++;
        }
    }

    int n = rand() % 5;//�������������
    if (n = 0)
        return tmppop;
    //���������ˣ����������ڵ�����������ݵ����������˳�����
    
    if(myList.size() > 0 && myList.size() >= n)
    {
        for (int i = 0; i < n; i++) {
            Student student = myList.front();
            //�ѳ����ݵ���Ա������popV��
            popV.push_back(student);
            myList.pop_front();//�Ƴ����ݵ���
            tmppop++;
        }
    }

    return tmppop;
}

void test()
{
    srand((unsigned int)time(NULL));
    list<Student> lift; //Ӣʽ����

    int PushNum = 0; //��¼�����ݵ�������
    int PopNum = 0; //��¼�����ݵ�������

    vector<Student> pushV;//��¼�����ݵ���Ա
    vector<Student> popV; //��¼�����ݵ���Ա

    //��������
    for (int i = 1; i <= 24; i++) {

        //������Ա
        queue<Student> que;
        //������Ա����
        CreateStudent(que, i);
        if (lift.size() <= 15)
        {
            //��24���¼�����,
            if (i < 24)
            {
                //������
                PushNum += PushList(lift, que, pushV);
            }
        }

        //�жϳ���������
        if (lift.size() > 0) //����Ҫ���˲��ܳ�
        {
            if (i > 1)  //1 ��ʱ, �����ǿյ�
            {
                // ������
                PopNum += PopList(lift, popV, i);
            }
        }
    }

    //��ӡ�����ݵ���Ա
    printVector(pushV);
    cout << "����������: " << PushNum << endl;
    // ��ӡ�����ݵ���Ա
    printVector(popV);
    cout << "����������: " << PopNum << endl;

}

int main()
{
    test();
    system("pause");
    return EXIT_SUCCESS;
}



