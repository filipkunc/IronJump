// Copyright 2010 Filip Kunc. All rights reserved.

var elevatorImages = new Array();
elevatorImages[0] = new Image();
elevatorImages[0].src = "Images/vytah01.png";
elevatorImages[1] = new Image();
elevatorImages[1].src = "Images/vytah02.png";
elevatorImages[2] = new Image();
elevatorImages[2].src = "Images/vytah03.png";

function FPElevator(x, y, endX, endY, widthSegments)
{
    this.x = x;
    this.y = y;
    this.startX = x;
    this.startY = y;
    this.endX = endX;
    this.endY = endY;
    this.widthSegments = widthSegments;
    this.textureIndex = 0;
    this.animationCounter = 0;
    this.movingToEnd = true;
    this.affectedObjects = null;
    this.isVisible = true;

    this.isPlatform = function()
    {
        return true;
    }
    
    this.isMovable = function()
    {
        return false;
    }
    
    this.rect = function()
    {
        return new FPRect(this.x, this.y, this.widthSegments * 32.0, 32.0);
    }
    
    this.move = function(offsetX, offsetY)
    {
        this.x += offsetX;
        this.y += offsetY;
        this.startX += offsetX;
        this.startY += offsetY;
        this.endX += offsetX;
        this.endY += offsetY;
    }
    
    this.moveCurrent = function(offsetX, offsetY)
    {
        this.x += offsetX;
        this.y += offsetY;
    }
    
    this.elevatorCollision = function(game, diffX, diffY)
    {
        var player = game.player;
        var playerRect = player.rect();
        var moveRect = FPRectWithMove(this.rect(), diffX, diffY);
        
        if (diffY > 0.0)
        {
            if (FPRectIntersectsRectWithTolerance(playerRect, moveRect))
                diffY = 0.0;
            else
            {
                for (i in game.gameObjects)
                {
                    var gameObject = game.gameObjects[i];
                    if (gameObject.isMovable())
                    {
                        if (FPRectIntersectsRectWithTolerance(gameObject.rect(), moveRect))
                        {
                            diffY = 0.0;
                            break;
                        }
                    }
                }
            }
        }
        
        if (Math.abs(diffX) > 0.0)
        {
            if (FPRectIntersectsRectWithTolerance(playerRect, moveRect))
                diffX = 0.0;
            else
            {   
                for (i in game.gameObjects)
                {
                    var gameObject = game.gameObjects[i];
                    if (gameObject.isMovable())
                    {
                        if (FPRectIntersectsRectWithTolerance(gameObject.rect(), moveRect))
                        {
                            diffX = 0.0;
                            break;
                        }
                    }
                }
            }
        }
        
        playerRect.size.height += tolerance;
        
        if (FPRectIntersectsRectWithTolerance(playerRect, moveRect))
        {
            game.moveWorld(-diffX, 0.0);
            player.collisionLeftRight(game);
            game.moveWorld(0.0, -diffY);
        }
        else
        {
            for (i in this.affectedObjects)
            {
                var gameObject = this.affectedObjects[i];
                moveRect = FPRectWithMove(gameObject.rect(), diffX, diffY);
                if (FPRectIntersectsRectWithTolerance(playerRect, moveRect))
                {
                    game.moveWorld(-diffX, 0.0);
                    player.collisionLeftRight(game);
                    game.moveWorld(0.0, -diffY);
                    break;
                }
            }
        }
        
        var movableOnElevator = null;
        var movableRect;
        
        for (i in game.gameObjects)
        {
            var movable = game.gameObjects[i];
            if (movable.isMovable())
            {
                moveRect = FPRectWithMove(this.rect(), diffX, diffY);
                movableRect = movable.rect();
                movableRect.size.height += tolerance;
                if (movableRect.bottom() < moveRect.bottom() &&
                    FPRectIntersectsRectWithTolerance(movableRect, moveRect))
                {
                    movableOnElevator = movable;
                    break;                        
                }
                else
                {
                    for (j in this.affectedObjects)
                    {
                        var gameObject = this.affectedObjects[i];
                        moveRect = FPRectWithMove(gameObject.rect(), diffX, diffY);
                        if (movableRect.bottom() < moveRect.bottom() &&
                            FPRectIntersectsRectWithTolerance(movableRect, moveRect))
                        {
                            movableOnElevator = movable;
                            break;                        
                        }
                    }
                }
            }
        }
        
        if (movableOnElevator != null)
        {
            var movableRect = FPRectWithMove(movableOnElevator.rect(), diffX, diffY);
            movableOnElevator.move(diffX, 0.0);
            if (FPRectIntersectsRectWithTolerance(playerRect, movableRect))
            {
                game.moveWorld(-diffX, 0.0);
                player.collisionLeftRight(game);
            }
            movableOnElevator.move(0.0, diffY);
            if (FPRectIntersectsRectWithTolerance(playerRect, movableRect))
            {
                game.moveWorld(0.0, -diffY);
            }
        }
        
        this.x += diffX;
        this.y += diffY;
        
        for (i in this.affectedObjects)
        {
            var gameObject = this.affectedObjects[i];
            gameObject.move(diffX, diffY);
        }
    }
    
    this.initAffectedObjectsIfNeeded = function(game)
    {
        if (this.affectedObjects == null)
        {
            this.affectedObjects = new Array();
            var selfRect = this.rect();
            
            for (i in game.gameObjects)
            {
                var gameObject = game.gameObjects[i];
                if (gameObject == this)
                    continue;
                    
                var gameObjectRect = gameObject.rect();
                gameObjectRect.size.height += tolerance;
                if (FPRectIntersectsRect(gameObjectRect, selfRect))
                    this.affectedObjects.push(gameObject);
            }
            
            for (i in game.gameObjects)
            {
                var gameObject = game.gameObjects[i];
                if (gameObject == this)
                    continue;
                    
                if (this.affectedObjects.contains(gameObject))
                    continue;
                    
                var gameObjectRect = gameObject.rect();
                gameObjectRect.size.height += tolerance;
                
                for (j in this.affectedObjects)
                {
                    if (FPRectIntersectsRect(gameObjectRect, this.affectedObjects[j].rect()))
                        this.affectedObjects.push(gameObject);
                }
            }
        }
    }
    
    this.update = function(game)
    {
        
    }
    
    this.draw = function(context)
    {
        for (iy = 0; iy < this.heightSegments; iy++)
        {
            for (ix = 0; ix < this.widthSegments; ix++)
            {
                context.drawImage(elevatorImages[this.textureIndex],
                    this.x + ix * 32.0, this.y + iy * 32.0);
            }
        }
    }
}