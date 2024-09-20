
#[cfg(test)]
mod tests {
    use crate::exports::{Component, query::{Exclude, Optional}, World};

    struct ZeroSized;

    impl Component for ZeroSized {}

    #[test]
    fn query_iter_zero_sized() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let ent = world.create_entity();
            world.add_component(ent, ZeroSized);
        }

        let query = world.query::<ZeroSized>().unwrap();

        let mut count = 0;

        for (_entity, _x) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_iter_mut_zero_sized() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let ent = world.create_entity();
            world.add_component(ent, ZeroSized);
        }

        let query = world.query_mut::<ZeroSized>().unwrap();

        let mut count = 0;

        for (_entity, _x) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_iter_mut() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }
        }

        let query = world.query_mut::<f32>().unwrap();

        let mut count = 0;

        for (_entity, component) in query {
            *component += count as f32;

            count += 1;
        }

        assert_eq!(count, 500);

        let query = world.query::<f32>().unwrap();

        let mut count = 0;
        let mut sum = 0.0;

        for (_entity, component) in query {
            count += 1;

            sum += *component;
        }

        assert_eq!(count, 500);
        assert_eq!(sum, 125250.0);
    }

    #[test]
    fn query_one() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }
        }

        let query = world.query::<f32>().unwrap();

        let mut count = 0;

        for (entity, component) in query {
            assert!(entity.id % 2 == 0);
            assert_eq!(component, &1.0f32);

            count += 1;
        }

        assert_eq!(count, 500);
    }

    #[test]
    fn query_two() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }
        }

        let query = world.query::<(f32, i32)>().unwrap();

        let mut count = 0;

        for (entity, (component1, component2)) in query {
            assert!(entity.id % 2 == 0);
            assert_eq!(component1, &1.0f32);
            assert_eq!(component2, &0);

            count += 1;
        }

        assert_eq!(count, 500);
    }

    #[test]
    fn query_optional() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query::<(i32, Optional<f32>)>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_ref());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_optional_only() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query::<Optional<f32>>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, component) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_ref());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_optional_unregistered() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world
            .query::<(i32, Optional<f32>, Optional<f64>)>()
            .unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component, _y)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_ref());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_mut_optional_unregistered() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world
            .query_mut::<(i32, Optional<f32>, Optional<f64>)>()
            .unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component, _y)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_mut());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_exclude_unregistered() {
        let mut world = World::with_capacity(1000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0i32);
        }

        let query = world.query::<(Exclude<f32>, i32)>().unwrap();

        let mut count = 0;

        for (_, (_x, _y)) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_mut_exclude_unregistered() {
        let mut world = World::with_capacity(1000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);
        }

        let query = world.query_mut::<(Exclude<f32>, i32)>().unwrap();

        let mut count = 0;

        for (_, (_x, _y)) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_mut_optional() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query_mut::<(i32, Optional<f32>)>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_mut());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_entity_optional_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 2i32);

        let data = world.query_entity::<Optional<i32>>(ent);

        if let Some(data) = data {
            assert_eq!(data, Some(&2i32));
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_optional_unregistered_component() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity::<(f32, Optional<i32>, Optional<f64>)>(ent);

        if let Some(data) = data {
            let (float, int, double) = data;

            assert_eq!(float, &0f32);
            assert_eq!(int, Some(&2i32));
            assert_eq!(double, None);
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_mut_optional_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<Optional<i32>>(ent);

        if let Some(data) = data {
            assert_eq!(data, Some(&mut 2i32));
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_mut_optional_unregistered_component() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<(f32, Optional<i32>, Optional<f64>)>(ent);

        if let Some(data) = data {
            let (float, int, double) = data;

            assert_eq!(float, &mut 0f32);
            assert_eq!(int, Some(&mut 2i32));
            assert_eq!(double, None);
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_exclude_unregistered() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 2i32);

        let data = world.query_entity::<(Exclude<f32>, i32)>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query was found");
        }
    }

    #[test]
    fn query_entity_exclude_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity::<Exclude<f32>>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query found");
        }
    }

    #[test]
    fn query_entity_mut_exclude_unregistered() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<(Exclude<f32>, i32)>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query found");
        }
    }

    #[test]
    fn query_entity_mut_exclude_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<Exclude<f32>>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query found");
        }
    }

    #[test]
    fn query_mut_optional_only() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query_mut::<Optional<f32>>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, component) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_mut());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_exclude() {
        let mut world = World::with_capacity(10000);

        let mut some_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count += 1;
            }
        }

        let query = world.query::<(i32, Exclude<f32>)>().unwrap();

        let mut query_count = 0;

        for (_, (_x, _exclude)) in query {
            query_count += 1;
        }

        assert_eq!(query_count, some_count);
    }

    #[test]
    fn query_mut_exclude() {
        let mut world = World::with_capacity(10000);

        let mut some_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count += 1;
            }
        }

        let query = world.query_mut::<(i32, Exclude<f32>)>().unwrap();

        let mut query_count = 0;

        for (_, (_x, _exclude)) in query {
            query_count += 1;
        }

        assert_eq!(query_count, some_count);
    }

    #[test]
    fn query_exclude_only() {
        let mut world = World::with_capacity(10000);

        let mut some_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count += 1;
            }
        }

        let query = world.query::<Exclude<f32>>().unwrap();

        let mut total_count = 0;

        for (_, _x) in query {
            total_count += 1;
        }

        assert_eq!(total_count, some_count);
    }

    #[test]
    fn query_mut_exclude_only() {
        let mut world = World::with_capacity(10000);

        let mut exclude_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                exclude_count += 1;
            }
        }

        let query = world.query_mut::<Exclude<f32>>().unwrap();

        let mut total_count = 0;

        for (_, _x) in query {
            total_count += 1;
        }

        assert_eq!(total_count, 1000 - exclude_count);
    }

    #[test]
    fn query_multiple() {
        let mut world = World::with_capacity(10000);

        for i in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }

            world.add_component(entity, i as f64);
        }

        let query = world.query::<(f32, i32, f64)>().unwrap();

        let mut count = 0;

        for (entity, (component1, component2, component3)) in query {
            assert!(entity.id % 2 == 0);
            assert_eq!(component1, &1.0f32);
            assert_eq!(component2, &0);
            assert_eq!(component3 % 2.0, 0.0);

            count += 1;
        }

        assert_eq!(count, 500);
    }

    #[test]
    fn query_entity() {
        let mut world = World::with_capacity(10000);

        let entities = (0..1000).map(|_| world.create_entity()).collect::<Vec<_>>();

        for i in 0..1000 {
            let entity = entities[i];

            world.add_component(entity, 0);

            world.add_component(entity, i as f64);
        }

        for i in 0..1000 {
            let entity = entities[i];

            let components = world.query_entity::<(i32, f64)>(entity).unwrap();

            if entity.id % 2 == 0 {
                assert_eq!(components, (&0, &(i as f64)));
            } else {
                assert_eq!(components, (&0, &(i as f64)));
            }
        }
    }

    #[test]
    fn query_entity_mut() {
        let mut world = World::with_capacity(10000);

        let entities = (0..1000).map(|_| world.create_entity()).collect::<Vec<_>>();

        for i in 0..1000 {
            let entity = entities[i];

            world.add_component(entity, 0);

            world.add_component(entity, i as f64);
        }

        for i in 0..1000 {
            let entity = entities[i];

            let components = world.query_entity_mut::<(i32, f64)>(entity).unwrap();

            if entity.id % 2 == 0 {
                assert_eq!(components, (&mut 0, &mut (i as f64)));
            } else {
                assert_eq!(components, (&mut 0, &mut (i as f64)));
            }

            *components.0 = 1;
            *components.1 = 1.0;
        }

        for i in 0..1000 {
            let entity = entities[i];

            let components = world.query_entity::<(i32, f64)>(entity).unwrap();

            assert_eq!(components, (&1, &1.0));
        }
    }
}
