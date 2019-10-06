select * FROM note where
        (
        title like '%rust%'
        or url like '%rust%'
        or tags like '%rust%'
        or description like '%rust%'
        )
        order by created_at;

delete FROM note where NOT
        (
        title like '%rust%'
        or url like '%rust%'
        or tags like '%rust%'
        or description like '%rust%'
        );
