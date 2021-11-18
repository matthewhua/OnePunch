//
// Created by Admin on 2021/11/15.
//

#include<iostream>
using namespace std;
#include<queue>
#include<string>

void test01()
{
    queue<int> queue;
    for (int i = 0; i < 5; ++i) {
        queue.push(i + 1);
    }
    cout << "front:" << queue.front() << endl;
    cout << "back:" << queue.back() << endl;

    while (!queue.empty())
    {
        cout << queue.front() << " ";
        queue.pop();
    }
    cout << endl;
    cout << queue.size() << endl;
}


class Maker
{
public:
    Maker(const string &name, int age) : name(name), age(age) {}

public:
    string name;
    int age;
};

void test02()
{
    queue<Maker *> q;
    q.push(new Maker("aaa", 18));
    q.push(new Maker("bbb", 19));
    q.push(new Maker("ccc", 20));

    while (!q.empty())
    {
        Maker *m = q.front();
        cout << m->name << " " << m-> age << endl;
        q.pop();
        delete m;
    }
    cout << q.size() << endl;

}

int main()
{
    test01();
    test02();
    system("pause");
    return EXIT_SUCCESS;
}
