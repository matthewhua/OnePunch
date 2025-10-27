package com.gameserver.dao.impl;

import com.gameserver.dao.BackpackDAO;
import com.gameserver.entity.Backpack;
import org.hibernate.Session;
import org.hibernate.SessionFactory;
import org.hibernate.query.Query;

import javax.inject.Inject;
import java.util.List;
import java.util.Optional;

/**
 * 背包 DAO 实现
 * 
 * 特点：
 * 1. 通过 Dagger2 注入 SessionFactory
 * 2. 使用 Hibernate ORM 进行数据操作
 * 3. 支持事务性操作
 * 4. 与 Pekko Actor 配合使用
 */
public class BackpackDAOImpl implements BackpackDAO {
    
    private final SessionFactory sessionFactory;
    
    @Inject
    public BackpackDAOImpl(SessionFactory sessionFactory) {
        this.sessionFactory = sessionFactory;
    }
    
    @Override
    public Optional<Backpack> findByPlayerAndItem(Long playerId, Long itemId) {
        try (Session session = sessionFactory.openSession()) {
            String hql = "FROM Backpack WHERE playerId = :playerId AND itemId = :itemId";
            Query<Backpack> query = session.createQuery(hql, Backpack.class);
            query.setParameter("playerId", playerId);
            query.setParameter("itemId", itemId);
            
            return query.uniqueResultOptional();
        }
    }
    
    @Override
    public List<Backpack> findByPlayerId(Long playerId) {
        try (Session session = sessionFactory.openSession()) {
            String hql = "FROM Backpack WHERE playerId = :playerId ORDER BY itemId";
            Query<Backpack> query = session.createQuery(hql, Backpack.class);
            query.setParameter("playerId", playerId);
            
            return query.getResultList();
        }
    }
    
    @Override
    public void save(Backpack backpack) {
        try (Session session = sessionFactory.openSession()) {
            org.hibernate.Transaction tx = session.beginTransaction();
            try {
                session.saveOrUpdate(backpack);
                tx.commit();
            } catch (Exception e) {
                tx.rollback();
                throw e;
            }
        }
    }
    
    @Override
    public void delete(Long backpackId) {
        try (Session session = sessionFactory.openSession()) {
            org.hibernate.Transaction tx = session.beginTransaction();
            try {
                Backpack backpack = session.get(Backpack.class, backpackId);
                if (backpack != null) {
                    session.delete(backpack);
                }
                tx.commit();
            } catch (Exception e) {
                tx.rollback();
                throw e;
            }
        }
    }
    
    @Override
    public void deleteByPlayerAndItem(Long playerId, Long itemId) {
        try (Session session = sessionFactory.openSession()) {
            org.hibernate.Transaction tx = session.beginTransaction();
            try {
                String hql = "DELETE FROM Backpack WHERE playerId = :playerId AND itemId = :itemId";
                Query<?> query = session.createQuery(hql);
                query.setParameter("playerId", playerId);
                query.setParameter("itemId", itemId);
                query.executeUpdate();
                tx.commit();
            } catch (Exception e) {
                tx.rollback();
                throw e;
            }
        }
    }
}
