package io.hibernate.orm.demo.entity;

import javax.persistence.*;

/**
 * The Class Community.
 */
@Entity
public class Community {
	
	/** The id. */
	@Id
	@GeneratedValue
	private int id;

	/** The name. */
	@Column(name = "name")
	private String name;

	/** The creator. */
	@ManyToOne
	private User creator = null;
}
