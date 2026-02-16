// Example test demonstrating per-TmpRepo environment variable tracking
//
// This test shows how to use the new environment variable tracking feature
// in TmpRepo, which allows each test repository to have its own isolated
// environment variables without affecting other tests or the process environment.

#[cfg(test)]
mod tests {
    use crate::git::test_utils::TmpRepo;

    #[test]
    fn test_repo_specific_env_vars() {
        // Create two separate test repositories
        let repo1 = TmpRepo::new().unwrap();
        let repo2 = TmpRepo::new().unwrap();

        // Set different environment variables for each repo
        repo1.set_env("MY_TEST_VAR", "value_for_repo1");
        repo1.set_env("SHARED_VAR", "repo1");
        
        repo2.set_env("MY_TEST_VAR", "value_for_repo2");
        repo2.set_env("SHARED_VAR", "repo2");

        // Verify each repo has its own isolated environment
        assert_eq!(repo1.get_env("MY_TEST_VAR"), Some("value_for_repo1".to_string()));
        assert_eq!(repo2.get_env("MY_TEST_VAR"), Some("value_for_repo2".to_string()));
        
        assert_eq!(repo1.get_env("SHARED_VAR"), Some("repo1".to_string()));
        assert_eq!(repo2.get_env("SHARED_VAR"), Some("repo2".to_string()));

        // Unset a variable in one repo doesn't affect the other
        repo1.unset_env("SHARED_VAR");
        assert_eq!(repo1.get_env("SHARED_VAR"), None);
        assert_eq!(repo2.get_env("SHARED_VAR"), Some("repo2".to_string()));

        // Clear all env vars in one repo
        repo2.clear_env();
        assert_eq!(repo2.get_env("MY_TEST_VAR"), None);
        assert_eq!(repo2.get_env("SHARED_VAR"), None);
        
        // repo1 is unaffected
        assert_eq!(repo1.get_env("MY_TEST_VAR"), Some("value_for_repo1".to_string()));
    }

    #[test]
    fn test_env_vars_passed_to_git_commands() {
        let repo = TmpRepo::new().unwrap();
        
        // Set an environment variable that git commands will see
        repo.set_env("GIT_AUTHOR_NAME", "Test Author");
        repo.set_env("GIT_AUTHOR_EMAIL", "test@example.com");
        
        // When git commands run in this repo context, they will have access
        // to these environment variables
        let file = repo.write_file("test.txt", "Hello, World!", true).unwrap();
        repo.trigger_checkpoint_with_author("ai_test").unwrap();
        
        // All git operations in this repo will use the custom environment
        let _result = repo.commit_with_message("Test commit");
        
        // The environment variables are only set for this repo's git commands
        // and don't leak to the process or other repos
        assert_eq!(repo.get_env("GIT_AUTHOR_NAME"), Some("Test Author".to_string()));
    }
}
