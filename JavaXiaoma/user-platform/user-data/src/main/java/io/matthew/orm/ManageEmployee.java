package io.matthew.orm;

import io.matthew.entity.Employee;
import org.hibernate.HibernateException;
import org.hibernate.Session;
import org.hibernate.Transaction;

import java.util.Iterator;
import java.util.List;

/**
 * @author Matthew
 * @date 2021-12-12 20:11
 */
public class ManageEmployee {

    /* Method to CREATE an employee in the database */
    public int addEmployee(String fname, String lname, int salary){
        final Session session = HibernateDemo.factory.openSession();
        Transaction tx = null;
        Integer employeeId = null;

        try {
            tx = session.beginTransaction();
            final Employee employee = new Employee(fname, lname, salary);
            employeeId = (Integer) session.save(employee);
            tx.commit();
        } catch (HibernateException e) {
            if (tx!=null) tx.rollback();
            e.printStackTrace();
        }finally {
            session.close();
        }
        return employeeId;
    }

    /* Method to  READ all the employees */
    public void listEmployees( ) {
        Session session = HibernateDemo.factory.openSession();
        Transaction tx = null;

        try {

            tx = session.beginTransaction();
            final List employees = session.createQuery("From Employee").list();
            for (Iterator iterator =
                 employees.iterator(); iterator.hasNext();){
                Employee employee = (Employee) iterator.next();
                System.out.print("First Name: " + employee.getFirstName());
                System.out.print("  Last Name: " + employee.getLastName());
                System.out.println("  Salary: " + employee.getSalary());
            }
           tx.commit();
        }catch (HibernateException e) {
            if (tx!=null) tx.rollback();
            e.printStackTrace();
        }finally {
            session.close();
        }
    }

    public void updateEmployee(Integer EmployeeId, int salary)
    {
        Session session = HibernateDemo.factory.openSession();
        Transaction tx = null;
        try {
            tx = session.beginTransaction();
            final Employee employee = session.get(Employee.class, EmployeeId);
            employee.setSalary(salary);
            session.update(employee);
            tx.commit();
        }catch (HibernateException e) {
            if (tx!=null) tx.rollback();
            e.printStackTrace();
        }finally {
            session.close();
        }
    }


    /* Method to DELETE an employee from the records */
    public void deleteEmployee(Integer EmployeeID){
        Session session = HibernateDemo.factory.openSession();
        Transaction tx = null;
        try{
            tx = session.beginTransaction();
            Employee employee =
                    (Employee)session.get(Employee.class, EmployeeID);
            session.delete(employee);
            tx.commit();
        }catch (HibernateException e) {
            if (tx!=null) tx.rollback();
            e.printStackTrace();
        }finally {
            session.close();
        }
    }

}
