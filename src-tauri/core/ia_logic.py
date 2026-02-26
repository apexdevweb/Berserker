import ollama

# On donne une identité à l'IA dès le départ
messages_historique = [
    {
        'role': 'system', 
        'content': "Tu es un assistant personnel intelligent. Tu dois te souvenir de tout ce que l'utilisateur te dit sur lui (nom, âge, goûts). Nous sommes en 2024."
    }
]

def demander_a_ia(prompt):
    global messages_historique
    try:
        messages_historique.append({'role': 'user', 'content': prompt})

        response = ollama.chat(
            model='llama3.2:latest',
            messages=messages_historique,
        )

        reponse_ia = response['message']['content']
        messages_historique.append({'role': 'assistant', 'content': reponse_ia})

        return reponse_ia
    except Exception as e:
        return f"Erreur : {str(e)}"

